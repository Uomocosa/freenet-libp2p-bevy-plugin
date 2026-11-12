use crate::starboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellCoverage {
    pub pos: starboard::canvas::CellPos,
    pub coverage: u8,
}

impl CellCoverage {
    #[must_use]
    pub(crate) fn from_inside_samples(pos: starboard::canvas::CellPos, inside: u8) -> Option<Self> {
        if inside == 0 {
            return None;
        }
        let scaled = u32::from(starboard::canvas::FULL_COVERAGE)
            .saturating_mul(u32::from(inside))
            .saturating_add(starboard::canvas::SAMPLES_PER_CELL / 2)
            / starboard::canvas::SAMPLES_PER_CELL;
        let coverage =
            u8::try_from(scaled.min(u32::from(starboard::canvas::FULL_COVERAGE))).ok()?;
        Some(Self { pos, coverage })
    }
}

#[cfg(test)]
mod tests {
    use super::CellCoverage;
    use crate::starboard::canvas::{CellPos, FULL_COVERAGE, SAMPLES_PER_CELL};

    #[test]
    fn test_usage() {
        let pos = CellPos { x: 1, y: 2 };
        assert_eq!(CellCoverage::from_inside_samples(pos, 0), None);
        let half = CellCoverage::from_inside_samples(pos, 8).map(|cell| cell.coverage);
        assert_eq!(half, Some(128));
        let full =
            CellCoverage::from_inside_samples(pos, u8::try_from(SAMPLES_PER_CELL).unwrap_or(0))
                .map(|cell| cell.coverage);
        assert_eq!(full, Some(FULL_COVERAGE));
    }
}
