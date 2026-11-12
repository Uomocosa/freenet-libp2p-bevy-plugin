use crate::starboard;

/// The first and last point of an op's polyline; a dab has the same point twice.
#[must_use]
pub fn op_ends(
    op: &starboard::canvas::DrawOp,
) -> (starboard::canvas::SubCellPos, starboard::canvas::SubCellPos) {
    (
        op.points.first().copied().unwrap_or_default(),
        op.points.last().copied().unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use super::op_ends;
    use crate::starboard::canvas::{DrawAction, DrawOp, SubCellPos};
    use crate::starboard::identity::Shape;

    #[test]
    fn test_usage() {
        let a = SubCellPos { x: 8, y: 8 };
        let b = SubCellPos { x: 40, y: 8 };
        let op = DrawOp {
            action: DrawAction::Erase,
            shape: Shape::Circle,
            size_index: 0,
            points: vec![a, b],
            stroke_rank: 1,
        };
        assert_eq!(op_ends(&op), (a, b));
    }
}
