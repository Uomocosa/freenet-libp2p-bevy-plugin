use crate::starboard;

/// Begins a stroke at `point`. The two earlier points repeat it, so the first segment starts
/// cleanly; nothing is emitted until a second point arrives.
pub const fn start_stroke(
    latest: &mut starboard::input::LatestPaintedCell,
    point: starboard::canvas::SubCellPos,
    action: starboard::canvas::DrawAction,
    stroke_rank: u32,
) {
    latest.cell = Some(point);
    latest.earlier_points = [point, point];
    latest.action = Some(action);
    latest.stroke_rank = stroke_rank;
}

#[cfg(test)]
mod tests {
    use super::start_stroke;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let mut latest = starboard::input::LatestPaintedCell::default();
        let point = starboard::canvas::SubCellPos { x: 8, y: 16 };
        start_stroke(&mut latest, point, starboard::canvas::DrawAction::Erase, 9);
        assert_eq!(latest.cell, Some(point));
        assert_eq!(latest.earlier_points, [point, point]);
        assert_eq!(latest.stroke_rank, 9);
    }
}
