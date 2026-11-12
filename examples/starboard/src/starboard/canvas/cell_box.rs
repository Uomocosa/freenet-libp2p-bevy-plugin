use crate::starboard;

/// An inclusive rectangle of canvas cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellBox {
    pub low_x: i64,
    pub high_x: i64,
    pub low_y: i64,
    pub high_y: i64,
}

impl CellBox {
    /// The cells whose centre lies within `reach` sub-cells of the segment `from -> to`, clipped
    /// to the canvas.
    #[must_use]
    pub fn around_segment(
        from: starboard::canvas::SubCellPos,
        to: starboard::canvas::SubCellPos,
        reach: i64,
    ) -> Self {
        let (from_x, from_y) = (i64::from(from.x), i64::from(from.y));
        let (to_x, to_y) = (i64::from(to.x), i64::from(to.y));
        let (low_x, high_x) = Self::axis_span(from_x.min(to_x), from_x.max(to_x), reach);
        let (low_y, high_y) = Self::axis_span(from_y.min(to_y), from_y.max(to_y), reach);
        Self {
            low_x,
            high_x,
            low_y,
            high_y,
        }
    }

    /// The smallest box containing both boxes.
    #[must_use]
    pub fn joined(&self, other: &Self) -> Self {
        Self {
            low_x: self.low_x.min(other.low_x),
            high_x: self.high_x.max(other.high_x),
            low_y: self.low_y.min(other.low_y),
            high_y: self.high_y.max(other.high_y),
        }
    }

    /// Whether the cell `(x, y)` lies inside the box.
    #[must_use]
    pub fn contains(&self, x: i64, y: i64) -> bool {
        (self.low_x..=self.high_x).contains(&x) && (self.low_y..=self.high_y).contains(&y)
    }

    /// The cell indices along one axis whose centre lies within `reach` of the span `[low, high]`.
    #[must_use]
    fn axis_span(low: i64, high: i64, reach: i64) -> (i64, i64) {
        let step = i64::from(starboard::canvas::SUBCELL_STEPS);
        let half = step / 2;
        let side = i64::try_from(starboard::canvas::CANVAS_SIDE).unwrap_or(0);
        let first = low
            .saturating_sub(reach)
            .saturating_sub(half)
            .saturating_add(step.saturating_sub(1));
        let last = high.saturating_add(reach).saturating_sub(half);
        let lo = first.div_euclid(step).max(0);
        let hi = last.div_euclid(step).min(side.saturating_sub(1));
        (lo, hi)
    }
}

#[cfg(test)]
mod tests {
    use super::CellBox;
    use crate::starboard::canvas::{CellPos, SubCellPos, cell_centre};

    #[test]
    fn test_usage() {
        let centre = cell_centre(CellPos { x: 100, y: 100 });
        let around = CellBox::around_segment(centre, centre, 8);
        assert!(around.contains(100, 100));
        assert!(!around.contains(90, 100));
        let edge = CellBox::around_segment(SubCellPos { x: 0, y: 0 }, SubCellPos { x: 0, y: 0 }, 4);
        assert_eq!(edge.low_x, 0);
        assert!(edge.contains(0, 0));
        assert_eq!(around.joined(&edge).low_x, 0);
    }
}
