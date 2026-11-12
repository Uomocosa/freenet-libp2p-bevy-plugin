use crate::starboard;

/// Ends the active stroke, returning the last segment with its final point repeated as the
/// lookahead, so the tail is not lost when the button is released.
#[must_use]
pub fn finish_stroke(
    latest: &mut starboard::input::LatestPaintedCell,
) -> Vec<Vec<starboard::canvas::SubCellPos>> {
    let Some(current) = latest.cell else {
        return Vec::new();
    };
    let [before, previous] = latest.earlier_points;
    let segment = if previous == current {
        Vec::new()
    } else {
        starboard::input::catmull_rom_points([before, previous, current, current])
    };
    latest.cell = None;
    latest.action = None;
    latest.earlier_points = [starboard::canvas::SubCellPos::default(); 2];
    starboard::canvas::DrawOp::chunk_points(segment)
}

#[cfg(test)]
mod tests {
    use super::finish_stroke;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let mut latest = starboard::input::LatestPaintedCell::default();
        let start = starboard::canvas::SubCellPos { x: 8, y: 8 };
        starboard::input::start_stroke(&mut latest, start, starboard::canvas::DrawAction::Erase, 1);
        assert_eq!(finish_stroke(&mut latest), Vec::<Vec<_>>::new());
        assert_eq!(latest.cell, None);
    }
}
