use crate::starboard;
use starboard::canvas::{
    CellBox, CellCoverage, CellPos, DrawOp, FULL_COVERAGE, HalfPlane, PairBins, SAMPLE_OFFSETS,
    SAMPLES_PER_CELL, SUBCELL_STEPS, SegmentSweep, SubCellPos,
};

const CENTRE_REACH_SLACK: i64 = 5;

pub struct StrokeRaster {
    pub shape: starboard::identity::Shape,
    pub sub_radius: u16,
    pub pieces: Vec<Vec<HalfPlane>>,
    pub pairs: Vec<(SubCellPos, SubCellPos)>,
    pub boxes: Vec<CellBox>,
    pub union: Option<CellBox>,
    pub bins: Option<PairBins>,
}

impl StrokeRaster {
    #[must_use]
    pub fn new(op: &DrawOp) -> Self {
        let radius = starboard::canvas::brush_radius(op.size_index);
        let sub_radius =
            u16::try_from(u32::from(radius).saturating_mul(SUBCELL_STEPS)).unwrap_or(u16::MAX);
        let kept = DrawOp::drop_collinear_points(&op.points);
        let pairs = DrawOp::point_pairs(&kept);
        let reach = i64::from(sub_radius).saturating_add(i64::from(SUBCELL_STEPS / 2));
        let boxes: Vec<CellBox> = pairs
            .iter()
            .map(|(from, to)| CellBox::around_segment(*from, *to, reach))
            .collect();
        let union = boxes
            .iter()
            .copied()
            .reduce(|left, right| left.joined(&right));
        let bins = union.map(|union_box| PairBins::new(&boxes, union_box));
        Self {
            shape: op.shape,
            sub_radius,
            pieces: starboard::canvas::shape_pieces(op.shape, sub_radius),
            pairs,
            boxes,
            union,
            bins,
        }
    }

    #[must_use]
    pub fn covered_cells(&self) -> Vec<CellCoverage> {
        let (Some(union), Some(bins)) = (self.union, self.bins.as_ref()) else {
            return Vec::new();
        };
        let mut covered = Vec::new();
        for x in union.low_x..=union.high_x {
            for y in union.low_y..=union.high_y {
                if let Some(coverage) = self.cell_coverage(bins, x, y) {
                    covered.push(coverage);
                }
            }
        }
        covered
    }

    pub(crate) fn cell_coverage(&self, bins: &PairBins, x: i64, y: i64) -> Option<CellCoverage> {
        let pos = CellPos {
            x: u16::try_from(x).unwrap_or(0),
            y: u16::try_from(y).unwrap_or(0),
        };
        let reachable: Vec<usize> = bins
            .candidates(x, y)
            .iter()
            .copied()
            .filter(|index| self.box_holds(*index, x, y) && self.centre_may_reach(*index, pos))
            .collect();
        if reachable.is_empty() {
            return None;
        }
        if reachable
            .iter()
            .any(|index| self.corners_inside_one_region(*index, pos))
        {
            return Some(CellCoverage {
                pos,
                coverage: FULL_COVERAGE,
            });
        }
        let inside = self.inside_sample_count(&reachable, pos);
        CellCoverage::from_inside_samples(pos, inside)
    }

    pub(crate) fn centre_may_reach(&self, index: usize, pos: CellPos) -> bool {
        let Some((from, to)) = self.pairs.get(index) else {
            return false;
        };
        let centre = starboard::canvas::cell_centre(pos);
        let sweep = SegmentSweep {
            start: (
                i64::from(centre.x).saturating_sub(i64::from(from.x)),
                i64::from(centre.y).saturating_sub(i64::from(from.y)),
            ),
            step: (
                i64::from(to.x).saturating_sub(i64::from(from.x)),
                i64::from(to.y).saturating_sub(i64::from(from.y)),
            ),
        };
        let radius = i64::from(self.sub_radius);
        let circumradius = if self.shape == starboard::identity::Shape::Circle {
            radius
        } else {
            radius.saturating_add(radius / 2)
        };
        sweep.meets_circle(circumradius.saturating_add(CENTRE_REACH_SLACK))
    }

    pub(crate) fn box_holds(&self, index: usize, x: i64, y: i64) -> bool {
        self.boxes
            .get(index)
            .is_some_and(|cell_box| cell_box.contains(x, y))
    }

    pub(crate) fn inside_sample_count(&self, reachable: &[usize], pos: CellPos) -> u8 {
        let origin = starboard::canvas::cell_centre(pos);
        let origin = SubCellPos {
            x: origin.x.saturating_sub(SUBCELL_STEPS / 2),
            y: origin.y.saturating_sub(SUBCELL_STEPS / 2),
        };
        let mut inside: u32 = 0;
        for across_x in SAMPLE_OFFSETS {
            for across_y in SAMPLE_OFFSETS {
                let point = SubCellPos {
                    x: origin.x.saturating_add(across_x),
                    y: origin.y.saturating_add(across_y),
                };
                if reachable
                    .iter()
                    .any(|index| self.sample_covered(*index, point))
                {
                    inside = inside.saturating_add(1);
                }
            }
        }
        u8::try_from(inside.min(SAMPLES_PER_CELL)).unwrap_or(0)
    }

    pub(crate) fn sample_covered(&self, index: usize, point: SubCellPos) -> bool {
        let Some((from, to)) = self.pairs.get(index) else {
            return false;
        };
        let offset = (
            i64::from(point.x).saturating_sub(i64::from(from.x)),
            i64::from(point.y).saturating_sub(i64::from(from.y)),
        );
        let step = (
            i64::from(to.x).saturating_sub(i64::from(from.x)),
            i64::from(to.y).saturating_sub(i64::from(from.y)),
        );
        starboard::canvas::segment_meets_shape(
            self.shape,
            self.sub_radius,
            &self.pieces,
            offset,
            step,
        )
    }

    pub(crate) fn corners_inside_one_region(&self, index: usize, pos: CellPos) -> bool {
        let Some((from, to)) = self.pairs.get(index) else {
            return false;
        };
        let step = (
            i64::from(to.x).saturating_sub(i64::from(from.x)),
            i64::from(to.y).saturating_sub(i64::from(from.y)),
        );
        let origin = starboard::canvas::cell_centre(pos);
        let origin = SubCellPos {
            x: origin.x.saturating_sub(SUBCELL_STEPS / 2),
            y: origin.y.saturating_sub(SUBCELL_STEPS / 2),
        };
        let [first, .., last] = SAMPLE_OFFSETS;
        let corners = [(first, first), (first, last), (last, first), (last, last)];
        let sweeps: [SegmentSweep; 4] = corners.map(|(across_x, across_y)| {
            let point = SubCellPos {
                x: origin.x.saturating_add(across_x),
                y: origin.y.saturating_add(across_y),
            };
            SegmentSweep {
                start: (
                    i64::from(point.x).saturating_sub(i64::from(from.x)),
                    i64::from(point.y).saturating_sub(i64::from(from.y)),
                ),
                step,
            }
        });
        if self.shape == starboard::identity::Shape::Circle {
            let radius = i64::from(self.sub_radius.max(1));
            return sweeps.iter().all(|sweep| sweep.meets_circle(radius));
        }
        self.pieces
            .iter()
            .any(|piece| sweeps.iter().all(|sweep| sweep.meets_piece(piece)))
    }
}

#[cfg(test)]
mod tests {
    use super::StrokeRaster;
    use crate::starboard::canvas::{CellPos, DrawAction, DrawOp, SubCellPos};
    use crate::starboard::identity::{PaletteColor, Shape};

    #[test]
    fn test_usage() {
        let op = DrawOp {
            action: DrawAction::Paint(PaletteColor(0)),
            shape: Shape::Circle,
            size_index: 2,
            points: vec![SubCellPos { x: 80, y: 80 }, SubCellPos { x: 160, y: 80 }],
            stroke_rank: 1,
        };
        let raster = StrokeRaster::new(&op);
        let covered = raster.covered_cells();
        assert!(
            covered
                .iter()
                .any(|cell| cell.pos == CellPos { x: 12, y: 10 })
        );
    }
}
