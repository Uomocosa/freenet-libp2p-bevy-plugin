use crate::starboard;

pub fn rasterize(
    op: &starboard::canvas::DrawOp,
) -> impl Iterator<Item = starboard::canvas::CellPos> {
    let radius = starboard::canvas::brush_radius(op.size_index);
    let offsets = shape_offsets(op.shape, radius);
    let mut cells = Vec::new();
    for center in stamp_centers(op.from, op.to) {
        let cx = i32::from(center.x);
        let cy = i32::from(center.y);
        for &(dx, dy) in &offsets {
            let x = cx.saturating_add(dx);
            let y = cy.saturating_add(dy);
            if let (Ok(ux), Ok(uy)) = (u16::try_from(x), u16::try_from(y))
                && usize::from(ux) < starboard::canvas::CANVAS_SIDE
                && usize::from(uy) < starboard::canvas::CANVAS_SIDE
            {
                cells.push(starboard::canvas::CellPos { x: ux, y: uy });
            }
        }
    }
    cells.sort_unstable();
    cells.dedup();
    cells.into_iter()
}

fn shape_offsets(shape: starboard::identity::Shape, radius: u16) -> Vec<(i32, i32)> {
    let r = i32::from(radius);
    let min_offset = 0_i32.saturating_sub(r);
    let mut offsets = Vec::new();
    for dy in min_offset..=r {
        for dx in min_offset..=r {
            if starboard::canvas::is_inside_shape(shape, radius, dx, dy) {
                offsets.push((dx, dy));
            }
        }
    }
    offsets
}

// needed helper: step along from -> to with integer Bresenham and emit a stamp centre on every cell
fn stamp_centers(
    from: starboard::canvas::CellPos,
    to: starboard::canvas::CellPos,
) -> Vec<starboard::canvas::CellPos> {
    let mut x = i32::from(from.x);
    let mut y = i32::from(from.y);
    let x1 = i32::from(to.x);
    let y1 = i32::from(to.y);
    let dx = (x1.saturating_sub(x)).abs();
    let sx = if x < x1 { 1_i32 } else { -1_i32 };
    let dy = 0_i32.saturating_sub((y1.saturating_sub(y)).abs());
    let sy = if y < y1 { 1_i32 } else { -1_i32 };
    let mut err = dx.saturating_add(dy);
    let mut centers = Vec::new();

    loop {
        if let (Ok(ux), Ok(uy)) = (u16::try_from(x), u16::try_from(y)) {
            centers.push(starboard::canvas::CellPos { x: ux, y: uy });
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = err.saturating_mul(2);
        if e2 >= dy {
            err = err.saturating_add(dy);
            x = x.saturating_add(sx);
        }
        if e2 <= dx {
            err = err.saturating_add(dx);
            y = y.saturating_add(sy);
        }
    }
    centers
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, VecDeque};

    use rstest::rstest;

    use super::{rasterize, shape_offsets, stamp_centers};
    use crate::starboard;
    use crate::starboard::canvas::{CANVAS_SIDE, CellPos, DrawAction, DrawOp, is_inside_shape};
    use crate::starboard::identity::{PaletteColor, SHAPES, Shape};

    #[test]
    fn test_usage() {
        for shape in SHAPES {
            let click_min_size = DrawOp {
                action: DrawAction::Paint(PaletteColor(0)),
                shape,
                size_index: 0,
                from: CellPos { x: 100, y: 100 },
                to: CellPos { x: 100, y: 100 },
                stroke_rank: 1,
            };
            let covered: BTreeSet<_> = rasterize(&click_min_size).collect();
            assert!(!covered.is_empty());
            assert!(covered.contains(&CellPos { x: 100, y: 100 }));
        }
    }

    #[rstest]
    #[case::circle_red(Shape::Circle, PaletteColor(0))]
    #[case::square_blue(Shape::Square, PaletteColor(6))]
    #[case::star_yellow(Shape::Star, PaletteColor(11))]
    fn single_dab_paints_inside_its_shape(#[case] shape: Shape, #[case] color: PaletteColor) {
        let center = CellPos { x: 100, y: 100 };
        let dab = DrawOp {
            action: DrawAction::Paint(color),
            shape,
            size_index: 0,
            from: center,
            to: center,
            stroke_rank: 1,
        };
        let radius = starboard::canvas::brush_radius(0);
        let covered: BTreeSet<_> = rasterize(&dab).collect();
        assert!(!covered.is_empty());
        assert!(covered.iter().all(|pos| {
            let dx = i32::from(pos.x).saturating_sub(i32::from(center.x));
            let dy = i32::from(pos.y).saturating_sub(i32::from(center.y));
            is_inside_shape(shape, radius, dx, dy)
        }));
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
            from: CellPos { x: 20, y: 20 },
            to: CellPos { x: 120, y: 20 },
            stroke_rank: 1,
        };
        let covered: BTreeSet<_> = rasterize(&stroke).collect();
        for x in 20_u16..=120 {
            assert!(
                covered.contains(&CellPos { x, y: 20 }),
                "missing centerline cell at x={x}"
            );
        }

        let max_coord = u16::try_from(CANVAS_SIDE.saturating_sub(1)).unwrap_or(1023);
        let border_op = DrawOp {
            action: DrawAction::Erase,
            shape,
            size_index: 8,
            from: CellPos { x: 0, y: 0 },
            to: CellPos { x: max_coord, y: 0 },
            stroke_rank: 1,
        };
        let border_cells: Vec<_> = rasterize(&border_op).collect();
        assert_ne!(border_cells, []);
        assert!(
            border_cells.iter().all(|pos| {
                usize::from(pos.x) < CANVAS_SIDE && usize::from(pos.y) < CANVAS_SIDE
            })
        );
    }

    const CURVE: [(u16, u16); 6] = [
        (20, 100),
        (60, 60),
        (100, 40),
        (140, 60),
        (180, 100),
        (200, 160),
    ];

    fn curve_cells(shape: Shape, color: PaletteColor) -> BTreeSet<CellPos> {
        let points: Vec<CellPos> = CURVE.iter().map(|&(x, y)| CellPos { x, y }).collect();
        let mut painted = BTreeSet::new();
        for pair in points.windows(2) {
            let op = DrawOp {
                action: DrawAction::Paint(color),
                shape,
                size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
                from: pair.first().copied().unwrap_or_default(),
                to: pair.last().copied().unwrap_or_default(),
                stroke_rank: 1,
            };
            let raw: Vec<CellPos> = rasterize(&op).collect();
            assert!(
                raw.iter().zip(raw.iter().skip(1)).all(|(a, b)| a < b),
                "one segment's output is strictly increasing, so it has no duplicates"
            );
            painted.extend(raw);
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
        let mut queue = VecDeque::from([start]);
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

    fn curve_region(shape: Shape, color: PaletteColor) -> BTreeSet<(i32, i32)> {
        curve_cells(shape, color)
            .iter()
            .map(|pos| (i32::from(pos.x), i32::from(pos.y)))
            .collect()
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
        let box_area = width.saturating_mul(height);
        let unpainted_in_box = box_area.saturating_sub(i32::try_from(cells.len()).unwrap_or(0));
        assert_eq!(
            i32::try_from(background.len()).unwrap_or(-1),
            unpainted_in_box,
            "every unpainted cell in the box reaches the border, so there are no holes"
        );
    }

    #[rstest]
    #[case::circle_red(Shape::Circle, PaletteColor(0))]
    #[case::square_blue(Shape::Square, PaletteColor(6))]
    #[case::star_yellow(Shape::Star, PaletteColor(11))]
    fn curved_stroke_sweeps_the_whole_brush_at_every_centre(
        #[case] shape: Shape,
        #[case] color: PaletteColor,
    ) {
        let painted = curve_region(shape, color);
        let radius = starboard::canvas::brush_radius(starboard::canvas::DEFAULT_SIZE_INDEX);
        let offsets = shape_offsets(shape, radius);
        let points: Vec<CellPos> = CURVE.iter().map(|&(x, y)| CellPos { x, y }).collect();
        for pair in points.windows(2) {
            for center in stamp_centers(pair[0], pair[1]) {
                let cx = i32::from(center.x);
                let cy = i32::from(center.y);
                for &(dx, dy) in &offsets {
                    assert!(
                        painted.contains(&(cx.saturating_add(dx), cy.saturating_add(dy))),
                        "the brush at {center:?} is missing offset ({dx}, {dy})"
                    );
                }
            }
        }
    }
}
