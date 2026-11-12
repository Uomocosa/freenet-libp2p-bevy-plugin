use crate::starboard;

pub fn rasterize(
    op: &starboard::canvas::DrawOp,
) -> impl Iterator<Item = starboard::canvas::CellCoverage> {
    starboard::canvas::StrokeRaster::new(op)
        .covered_cells()
        .into_iter()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use rstest::rstest;

    use super::rasterize;
    use crate::starboard;
    use crate::starboard::canvas::{
        CANVAS_SIDE, CellCoverage, CellPos, DrawAction, DrawOp, SAMPLE_OFFSETS, SUBCELL_STEPS,
        SubCellPos, cell_centre,
    };
    use crate::starboard::identity::{PaletteColor, SHAPES, Shape};

    fn sub(x: u32, y: u32) -> SubCellPos {
        SubCellPos { x, y }
    }

    fn op(shape: Shape, size_index: u8, from: SubCellPos, to: SubCellPos) -> DrawOp {
        DrawOp {
            action: DrawAction::Paint(PaletteColor(0)),
            shape,
            size_index,
            points: vec![from, to],
            stroke_rank: 1,
        }
    }

    fn point_inside(shape: Shape, radius: u16, dx: i64, dy: i64) -> bool {
        let to_i32 = |value: i64| i32::try_from(value).unwrap_or(i32::MAX);
        starboard::canvas::is_inside_shape(shape, radius, to_i32(dx), to_i32(dy))
    }

    /// Brute force: sample the segment every 1/128 cell and count, per cell, how many of its 16
    /// sub-samples the brush covers at some sample. Works in units of 1/256 cell, so it stays integer.
    fn dense_reference(op: &DrawOp) -> Vec<CellCoverage> {
        const UNITS_PER_CELL: i64 = 256;
        const UNITS_PER_SUB: i64 = 32;
        // 1/32 misses the sliver at a star vertex; 1/128 is the densest that stays fast enough
        const SAMPLE_UNITS: i64 = 2;
        let radius = i64::from(starboard::canvas::brush_radius(op.size_index));
        let unit_radius = u16::try_from(radius.saturating_mul(UNITS_PER_CELL)).unwrap_or(u16::MAX);
        let from = scaled_point(op.points.first().copied().unwrap_or_default());
        let to = scaled_point(op.points.last().copied().unwrap_or_default());
        let (dx, dy) = (to.0.saturating_sub(from.0), to.1.saturating_sub(from.1));
        let samples = dx
            .abs()
            .max(dy.abs())
            .checked_div(SAMPLE_UNITS)
            .unwrap_or(0)
            .max(1);
        let side = i64::try_from(CANVAS_SIDE).unwrap_or(0).saturating_sub(1);
        let cell_span = |low: i64, high: i64| {
            let reach = radius.saturating_add(2);
            let first = low
                .checked_div(UNITS_PER_CELL)
                .unwrap_or(0)
                .saturating_sub(reach);
            let last = high
                .checked_div(UNITS_PER_CELL)
                .unwrap_or(0)
                .saturating_add(reach);
            first.max(0)..=last.min(side)
        };
        let x_range = cell_span(from.0.min(to.0), from.0.max(to.0));
        let y_range = cell_span(from.1.min(to.1), from.1.max(to.1));
        let mut covered = Vec::new();
        for x in x_range {
            for y in y_range.clone() {
                let origin = (
                    x.saturating_mul(UNITS_PER_CELL),
                    y.saturating_mul(UNITS_PER_CELL),
                );
                let mut inside: u8 = 0;
                for across_x in SAMPLE_OFFSETS {
                    for across_y in SAMPLE_OFFSETS {
                        let sample = (
                            origin
                                .0
                                .saturating_add(i64::from(across_x).saturating_mul(UNITS_PER_SUB)),
                            origin
                                .1
                                .saturating_add(i64::from(across_y).saturating_mul(UNITS_PER_SUB)),
                        );
                        let hit = (0..=samples).any(|k| {
                            let along = |start: i64, delta: i64| {
                                start.saturating_add(
                                    delta.saturating_mul(k).checked_div(samples).unwrap_or(0),
                                )
                            };
                            let (px, py) = (along(from.0, dx), along(from.1, dy));
                            point_inside(
                                op.shape,
                                unit_radius,
                                sample.0.saturating_sub(px),
                                sample.1.saturating_sub(py),
                            )
                        });
                        if hit {
                            inside = inside.saturating_add(1);
                        }
                    }
                }
                let pos = CellPos {
                    x: u16::try_from(x).unwrap_or(0),
                    y: u16::try_from(y).unwrap_or(0),
                };
                covered.extend(CellCoverage::from_inside_samples(pos, inside));
            }
        }
        covered
    }

    fn scaled_point(point: SubCellPos) -> (i64, i64) {
        let units_per_sub = 256_i64.checked_div(i64::from(SUBCELL_STEPS)).unwrap_or(1);
        (
            i64::from(point.x).saturating_mul(units_per_sub),
            i64::from(point.y).saturating_mul(units_per_sub),
        )
    }

    /// Micro-benchmark, not a test: the same 40-cell line as one 2-point op and as a 160-point
    /// polyline, both at the big brush. Run with `--run-ignored only -- --nocapture`.
    #[test]
    #[ignore = "micro-benchmark: prints timings; run with --run-ignored only"]
    fn rasterize_straight_vs_polyline_micro_bench() {
        const ROUNDS: u32 = 200;
        const BIG_BRUSH: u8 = 7;
        const POLYLINE_POINTS: u32 = 160;
        let start = cell_centre(CellPos { x: 100, y: 100 });
        let end = cell_centre(CellPos { x: 140, y: 100 });
        let straight = op(Shape::Circle, BIG_BRUSH, start, end);
        let polyline = DrawOp {
            action: DrawAction::Paint(PaletteColor(0)),
            shape: Shape::Circle,
            size_index: BIG_BRUSH,
            points: (0..POLYLINE_POINTS)
                .map(|index| SubCellPos {
                    x: start.x + (end.x - start.x) * index / (POLYLINE_POINTS - 1),
                    y: start.y,
                })
                .collect(),
            stroke_rank: 1,
        };
        let curve = DrawOp {
            action: DrawAction::Paint(PaletteColor(0)),
            shape: Shape::Circle,
            size_index: BIG_BRUSH,
            points: starboard::input::catmull_rom_points(
                [(20, 100), (60, 60), (100, 40), (140, 60)]
                    .map(|(x, y)| cell_centre(CellPos { x, y })),
            ),
            stroke_rank: 1,
        };
        let time_rounds = |stroke: &DrawOp| {
            let started = std::time::Instant::now();
            for _ in 0..ROUNDS {
                std::hint::black_box(rasterize(stroke).count());
            }
            started.elapsed()
        };
        let straight_total = time_rounds(&straight);
        let polyline_total = time_rounds(&polyline);
        let curve_total = time_rounds(&curve);
        println!(
            "rasterize micro-bench ({ROUNDS} rounds, size index {BIG_BRUSH}): \
             (a) straight 2-point op {straight_total:?} ({} cells), \
             (b) 160-point polyline {polyline_total:?} ({} cells), \
             (c) curved spline of {} points {curve_total:?} ({} cells)",
            rasterize(&straight).count(),
            rasterize(&polyline).count(),
            curve.points.len(),
            rasterize(&curve).count(),
        );
    }

    #[test]
    fn test_usage() {
        for shape in SHAPES {
            let click = op(
                shape,
                0,
                cell_centre(CellPos { x: 100, y: 100 }),
                cell_centre(CellPos { x: 100, y: 100 }),
            );
            let covered: BTreeSet<_> = rasterize(&click).map(|cell| cell.pos).collect();
            assert!(covered.contains(&CellPos { x: 100, y: 100 }));
        }
    }

    #[rstest]
    #[case::circle_red(Shape::Circle, PaletteColor(0))]
    #[case::square_blue(Shape::Square, PaletteColor(6))]
    #[case::star_yellow(Shape::Star, PaletteColor(11))]
    fn single_dab_paints_inside_its_shape(#[case] shape: Shape, #[case] color: PaletteColor) {
        let centre = cell_centre(CellPos { x: 100, y: 100 });
        let dab = DrawOp {
            action: DrawAction::Paint(color),
            shape,
            size_index: 0,
            points: vec![centre, centre],
            stroke_rank: 1,
        };
        let covered: Vec<CellCoverage> = rasterize(&dab).collect();
        assert_ne!(covered, []);
        let radius = starboard::canvas::brush_radius(0).saturating_mul(8);
        for cell in &covered {
            let inside = sample_inside_count(shape, radius, centre, cell.pos);
            assert!(inside >= 1, "every emitted cell has an inside sample");
            assert!(
                cell.coverage != 255 || inside == 16,
                "255 means all samples inside"
            );
        }
    }

    fn sample_inside_count(shape: Shape, radius: u16, centre: SubCellPos, pos: CellPos) -> usize {
        let origin_x = u32::from(pos.x).saturating_mul(SUBCELL_STEPS);
        let origin_y = u32::from(pos.y).saturating_mul(SUBCELL_STEPS);
        let mut inside: usize = 0;
        for across_x in SAMPLE_OFFSETS {
            for across_y in SAMPLE_OFFSETS {
                let dx = i64::from(origin_x.saturating_add(across_x))
                    .saturating_sub(i64::from(centre.x));
                let dy = i64::from(origin_y.saturating_add(across_y))
                    .saturating_sub(i64::from(centre.y));
                if point_inside(shape, radius, dx, dy) {
                    inside = inside.saturating_add(1);
                }
            }
        }
        inside
    }

    #[rstest]
    #[case::circle_red(Shape::Circle, PaletteColor(0))]
    #[case::square_blue(Shape::Square, PaletteColor(6))]
    #[case::star_yellow(Shape::Star, PaletteColor(11))]
    fn segment_has_no_gaps_and_clips_at_canvas_borders(
        #[case] shape: Shape,
        #[case] color: PaletteColor,
    ) {
        let stroke = DrawOp {
            action: DrawAction::Paint(color),
            shape,
            size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
            points: vec![
                cell_centre(CellPos { x: 20, y: 20 }),
                cell_centre(CellPos { x: 120, y: 20 }),
            ],
            stroke_rank: 1,
        };
        let covered: BTreeSet<_> = rasterize(&stroke).map(|cell| cell.pos).collect();
        for x in 20_u16..=120 {
            assert!(
                covered.contains(&CellPos { x, y: 20 }),
                "missing centerline cell at x={x}"
            );
        }

        let max_coord = u32::try_from(CANVAS_SIDE.saturating_sub(1)).unwrap_or(1023);
        let border = DrawOp {
            action: DrawAction::Erase,
            shape,
            size_index: 8,
            points: vec![sub(0, 0), sub(max_coord.saturating_mul(SUBCELL_STEPS), 0)],
            stroke_rank: 1,
        };
        let border_cells: Vec<_> = rasterize(&border).map(|cell| cell.pos).collect();
        assert_ne!(border_cells, []);
        assert!(
            border_cells.iter().all(|pos| {
                usize::from(pos.x) < CANVAS_SIDE && usize::from(pos.y) < CANVAS_SIDE
            })
        );
    }

    #[rstest]
    #[case::circle_red(Shape::Circle, PaletteColor(0))]
    #[case::square_blue(Shape::Square, PaletteColor(6))]
    #[case::star_yellow(Shape::Star, PaletteColor(11))]
    fn diagonal_stroke_matches_the_dense_reference(
        #[case] shape: Shape,
        #[case] color: PaletteColor,
    ) {
        let stroke = DrawOp {
            action: DrawAction::Paint(color),
            shape,
            size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
            points: vec![sub(80, 80), sub(482, 303)],
            stroke_rank: 1,
        };
        let painted: Vec<CellCoverage> = rasterize(&stroke).collect();
        let reference = dense_reference(&stroke);
        let painted_cells: Vec<CellPos> = painted.iter().map(|cell| cell.pos).collect();
        let reference_cells: Vec<CellPos> = reference.iter().map(|cell| cell.pos).collect();
        assert_eq!(painted_cells, reference_cells);
        for (exact, sampled) in painted.iter().zip(reference.iter()) {
            let difference = i32::from(exact.coverage).abs_diff(i32::from(sampled.coverage));
            assert!(
                difference <= 16,
                "{:?}: exact {} vs sampled {}; the path sampling may miss one sub-sample on a sliver",
                exact.pos,
                exact.coverage,
                sampled.coverage
            );
        }
    }

    /// A spline stroke, flattened as the drawer sends it, paints exactly the cells that its
    /// segments paint one pair at a time, so skipping shared cells and dropping collinear points
    /// changes nothing visible.
    #[rstest]
    #[case::circle_red(Shape::Circle, PaletteColor(0))]
    #[case::square_blue(Shape::Square, PaletteColor(6))]
    #[case::star_yellow(Shape::Star, PaletteColor(11))]
    fn curved_spline_paints_the_same_cells_as_its_pairs(
        #[case] shape: Shape,
        #[case] color: PaletteColor,
    ) {
        let controls =
            [(20, 100), (60, 60), (100, 40), (140, 60)].map(|(x, y)| cell_centre(CellPos { x, y }));
        let points = starboard::input::catmull_rom_points(controls);
        let stroke = DrawOp {
            action: DrawAction::Paint(color),
            shape,
            size_index: 7,
            points: points.clone(),
            stroke_rank: 1,
        };
        let painted: BTreeSet<_> = rasterize(&stroke).map(|cell| cell.pos).collect();
        let reference: BTreeSet<_> = starboard::canvas::DrawOp::point_pairs(&points)
            .into_iter()
            .flat_map(|(from, to)| {
                let pair = DrawOp {
                    points: vec![from, to],
                    ..stroke.clone()
                };
                rasterize(&pair).map(|cell| cell.pos).collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(painted, reference);
    }

    const CURVE: [(u32, u32); 6] = [
        (20, 100),
        (60, 60),
        (100, 40),
        (140, 60),
        (180, 100),
        (200, 160),
    ];

    fn curve_region(shape: Shape, color: PaletteColor) -> BTreeSet<(i32, i32)> {
        let points: Vec<SubCellPos> = CURVE
            .iter()
            .map(|&(x, y)| {
                cell_centre(CellPos {
                    x: u16::try_from(x).unwrap_or(0),
                    y: u16::try_from(y).unwrap_or(0),
                })
            })
            .collect();
        let mut painted = BTreeSet::new();
        for pair in points.windows(2) {
            let stroke = DrawOp {
                action: DrawAction::Paint(color),
                shape,
                size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
                points: vec![
                    pair.first().copied().unwrap_or_default(),
                    pair.last().copied().unwrap_or_default(),
                ],
                stroke_rank: 1,
            };
            painted.extend(
                rasterize(&stroke).map(|cell| (i32::from(cell.pos.x), i32::from(cell.pos.y))),
            );
        }
        painted
    }

    const FOUR_STEPS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    const EIGHT_STEPS: [(i32, i32); 8] = [
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
    ];

    fn flood(
        start: (i32, i32),
        steps: &[(i32, i32)],
        is_open: impl Fn((i32, i32)) -> bool,
    ) -> BTreeSet<(i32, i32)> {
        let mut seen = BTreeSet::new();
        let mut queue = std::collections::VecDeque::from([start]);
        while let Some(cell) = queue.pop_front() {
            if !seen.insert(cell) {
                continue;
            }
            for &(dx, dy) in steps {
                let next = (cell.0.saturating_add(dx), cell.1.saturating_add(dy));
                if is_open(next) && !seen.contains(&next) {
                    queue.push_back(next);
                }
            }
        }
        seen
    }

    fn box_span(cells: &BTreeSet<(i32, i32)>, axis: fn(&(i32, i32)) -> i32) -> (i32, i32) {
        let low = cells
            .iter()
            .map(axis)
            .min()
            .unwrap_or_default()
            .saturating_sub(1);
        let high = cells
            .iter()
            .map(axis)
            .max()
            .unwrap_or_default()
            .saturating_add(1);
        (low, high)
    }

    /// The star's spike tips touch their neighbours only diagonally, even as a single dab,
    /// so the star ribbon is joined by 8-neighbours.
    #[rstest]
    #[case::circle_red(Shape::Circle, PaletteColor(0), &FOUR_STEPS)]
    #[case::square_blue(Shape::Square, PaletteColor(6), &FOUR_STEPS)]
    #[case::star_yellow(Shape::Star, PaletteColor(11), &EIGHT_STEPS)]
    fn curved_stroke_is_one_connected_ribbon(
        #[case] shape: Shape,
        #[case] color: PaletteColor,
        #[case] joins: &[(i32, i32)],
    ) {
        let cells = curve_region(shape, color);
        let first = cells.first().copied().unwrap_or_default();
        let reached = flood(first, joins, |cell| cells.contains(&cell));
        assert_eq!(
            reached.len(),
            cells.len(),
            "the stroke is one connected region"
        );
    }

    #[rstest]
    #[case::circle_red(Shape::Circle, PaletteColor(0))]
    #[case::square_blue(Shape::Square, PaletteColor(6))]
    fn curved_stroke_leaves_no_holes(#[case] shape: Shape, #[case] color: PaletteColor) {
        let cells = curve_region(shape, color);
        let (min_x, max_x) = box_span(&cells, |cell| cell.0);
        let (min_y, max_y) = box_span(&cells, |cell| cell.1);
        let is_open_in_box = |cell: (i32, i32)| {
            (min_x..=max_x).contains(&cell.0)
                && (min_y..=max_y).contains(&cell.1)
                && !cells.contains(&cell)
        };
        let background = flood((min_x, min_y), EIGHT_STEPS.as_slice(), is_open_in_box);
        let width = max_x.saturating_sub(min_x).saturating_add(1);
        let height = max_y.saturating_sub(min_y).saturating_add(1);
        let unpainted_in_box = width
            .saturating_mul(height)
            .saturating_sub(i32::try_from(cells.len()).unwrap_or(0));
        assert_eq!(
            i32::try_from(background.len()).unwrap_or(-1),
            unpainted_in_box,
            "every unpainted cell in the box reaches the border, so there are no holes"
        );
    }
}
