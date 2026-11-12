use crate::starboard;

/// Records the next cursor point of the active stroke. The segment between the previous two
/// points is flattened and returned once this point exists, which is the spline's lookahead.
#[must_use]
pub fn advance_stroke(
    latest: &mut starboard::input::LatestPaintedCell,
    point: starboard::canvas::SubCellPos,
) -> Vec<Vec<starboard::canvas::SubCellPos>> {
    let Some(current) = latest.cell else {
        return Vec::new();
    };
    if current == point {
        return Vec::new();
    }
    let [before, previous] = latest.earlier_points;
    let segment = if previous == current {
        Vec::new()
    } else {
        starboard::input::catmull_rom_points([before, previous, current, point])
    };
    latest.earlier_points = [previous, current];
    latest.cell = Some(point);
    starboard::input::chunk_points(segment)
}

#[cfg(test)]
mod tests {
    use super::advance_stroke;
    use crate::starboard;

    fn at(x: u32, y: u32) -> starboard::canvas::SubCellPos {
        starboard::canvas::SubCellPos { x, y }
    }

    #[test]
    fn test_usage() {
        let mut latest = starboard::input::LatestPaintedCell::default();
        starboard::input::start_stroke(
            &mut latest,
            at(0, 0),
            starboard::canvas::DrawAction::Erase,
            1,
        );
        assert_eq!(advance_stroke(&mut latest, at(0, 0)), Vec::<Vec<_>>::new());
        assert_eq!(advance_stroke(&mut latest, at(80, 0)), Vec::<Vec<_>>::new());
        let emitted = advance_stroke(&mut latest, at(160, 0));
        assert_eq!(emitted.len(), 1);
        assert_eq!(
            emitted.first().and_then(|piece| piece.first()),
            Some(&at(0, 0))
        );
        assert_eq!(
            emitted.first().and_then(|piece| piece.last()),
            Some(&at(80, 0))
        );
    }
}
