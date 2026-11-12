use crate::starboard;

#[must_use]
pub fn cell_centre(cell: starboard::canvas::CellPos) -> starboard::canvas::SubCellPos {
    let step = starboard::canvas::SUBCELL_STEPS;
    let half = step / 2;
    starboard::canvas::SubCellPos {
        x: u32::from(cell.x).saturating_mul(step).saturating_add(half),
        y: u32::from(cell.y).saturating_mul(step).saturating_add(half),
    }
}

#[cfg(test)]
mod tests {
    use super::cell_centre;
    use crate::starboard;

    #[test]
    fn test_usage() {
        assert_eq!(
            cell_centre(starboard::canvas::CellPos { x: 0, y: 2 }),
            starboard::canvas::SubCellPos { x: 4, y: 20 }
        );
    }
}
