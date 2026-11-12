use crate::starboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellCoverage {
    pub pos: starboard::canvas::CellPos,
    pub mask: u16,
}

impl CellCoverage {
    #[must_use]
    pub(crate) const fn from_mask(pos: starboard::canvas::CellPos, mask: u16) -> Option<Self> {
        if mask == 0 {
            return None;
        }
        Some(Self { pos, mask })
    }
}

#[cfg(test)]
mod tests {
    use super::CellCoverage;
    use crate::starboard::canvas::{CellPos, FULL_MASK};

    #[test]
    fn test_usage() {
        let pos = CellPos { x: 1, y: 2 };
        assert_eq!(CellCoverage::from_mask(pos, 0), None);
        assert_eq!(
            CellCoverage::from_mask(pos, FULL_MASK).map(|cell| cell.mask),
            Some(FULL_MASK)
        );
    }
}
