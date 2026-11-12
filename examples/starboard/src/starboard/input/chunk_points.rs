use crate::starboard;

/// Longest point list one op carries, so a fast flick stays far below the gossip frame cap.
pub const MAX_POINTS_PER_OP: usize = 512;

/// Splits a polyline into lists of at most `MAX_POINTS_PER_OP` points. Neighbouring lists share
/// their boundary point, so the pieces rasterize as one connected stroke.
#[must_use]
pub fn chunk_points(
    points: Vec<starboard::canvas::SubCellPos>,
) -> Vec<Vec<starboard::canvas::SubCellPos>> {
    if points.len() <= MAX_POINTS_PER_OP {
        return if points.is_empty() {
            Vec::new()
        } else {
            vec![points]
        };
    }
    let stride = MAX_POINTS_PER_OP.saturating_sub(1);
    (0..points.len().saturating_sub(1))
        .step_by(stride)
        .filter_map(|start| {
            let end = start.saturating_add(MAX_POINTS_PER_OP).min(points.len());
            points.get(start..end).map(<[_]>::to_vec)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{MAX_POINTS_PER_OP, chunk_points};
    use crate::starboard::canvas::SubCellPos;

    fn line(count: u32) -> Vec<SubCellPos> {
        (0..count).map(|x| SubCellPos { x, y: 0 }).collect()
    }

    #[test]
    fn test_usage() {
        assert_eq!(chunk_points(Vec::new()), Vec::<Vec<SubCellPos>>::new());
        assert_eq!(chunk_points(line(3)), vec![line(3)]);
        let count = u32::try_from(MAX_POINTS_PER_OP.saturating_mul(3)).unwrap_or(0);
        let pieces = chunk_points(line(count));
        assert!(pieces.iter().all(|piece| piece.len() <= MAX_POINTS_PER_OP));
        assert_eq!(
            pieces.first().and_then(|piece| piece.last()),
            pieces.get(1).and_then(|piece| piece.first())
        );
    }
}
