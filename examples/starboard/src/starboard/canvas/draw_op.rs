use serde::{Deserialize, Serialize};

use crate::starboard;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawOp {
    pub action: starboard::canvas::DrawAction,
    pub shape: starboard::identity::Shape,
    pub size_index: u8,
    pub points: Vec<starboard::canvas::SubCellPos>,
    pub stroke_rank: u32,
}

impl DrawOp {
    #[must_use]
    pub(crate) fn drop_collinear_points(
        points: &[starboard::canvas::SubCellPos],
    ) -> Vec<starboard::canvas::SubCellPos> {
        let mut kept: Vec<starboard::canvas::SubCellPos> = Vec::with_capacity(points.len());
        for &point in points {
            if kept.last() == Some(&point) {
                continue;
            }
            while let [.., before, middle] = kept.as_slice() {
                let first = (
                    i64::from(middle.x).saturating_sub(i64::from(before.x)),
                    i64::from(middle.y).saturating_sub(i64::from(before.y)),
                );
                let second = (
                    i64::from(point.x).saturating_sub(i64::from(middle.x)),
                    i64::from(point.y).saturating_sub(i64::from(middle.y)),
                );
                let is_straight =
                    first.0.saturating_mul(second.1) == first.1.saturating_mul(second.0);
                let is_forward = first
                    .0
                    .saturating_mul(second.0)
                    .saturating_add(first.1.saturating_mul(second.1))
                    > 0;
                if !(is_straight && is_forward) {
                    break;
                }
                kept.pop();
            }
            kept.push(point);
        }
        kept
    }

    #[must_use]
    pub(crate) fn point_pairs(
        points: &[starboard::canvas::SubCellPos],
    ) -> Vec<(starboard::canvas::SubCellPos, starboard::canvas::SubCellPos)> {
        let pairs: Vec<_> = points
            .iter()
            .zip(points.iter().skip(1))
            .map(|(from, to)| (*from, *to))
            .collect();
        match (pairs.is_empty(), points.first()) {
            (true, Some(only)) => vec![(*only, *only)],
            _ => pairs,
        }
    }

    #[must_use]
    pub(crate) fn chunk_points(
        points: Vec<starboard::canvas::SubCellPos>,
    ) -> Vec<Vec<starboard::canvas::SubCellPos>> {
        if points.len() <= starboard::canvas::MAX_POINTS_PER_OP {
            return if points.is_empty() {
                Vec::new()
            } else {
                vec![points]
            };
        }
        let stride = starboard::canvas::MAX_POINTS_PER_OP.saturating_sub(1);
        (0..points.len().saturating_sub(1))
            .step_by(stride)
            .filter_map(|start| {
                let end = start
                    .saturating_add(starboard::canvas::MAX_POINTS_PER_OP)
                    .min(points.len());
                points.get(start..end).map(<[_]>::to_vec)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::DrawOp;
    use crate::starboard::canvas::{MAX_POINTS_PER_OP, SubCellPos};

    fn at(x: u32, y: u32) -> SubCellPos {
        SubCellPos { x, y }
    }

    fn line(count: u32) -> Vec<SubCellPos> {
        (0..count).map(|x| SubCellPos { x, y: 0 }).collect()
    }

    #[test]
    fn test_usage() {
        assert_eq!(
            DrawOp::drop_collinear_points(&[at(0, 0), at(4, 0), at(8, 0), at(8, 5)]),
            vec![at(0, 0), at(8, 0), at(8, 5)]
        );
        assert_eq!(
            DrawOp::drop_collinear_points(&[at(0, 0), at(4, 0), at(2, 0)]),
            vec![at(0, 0), at(4, 0), at(2, 0)]
        );
        assert_eq!(
            DrawOp::drop_collinear_points(&[at(1, 1), at(1, 1)]),
            vec![at(1, 1)]
        );
        assert_eq!(DrawOp::drop_collinear_points(&[]), Vec::new());

        let (a, b, c) = (at(1, 1), at(2, 1), at(3, 1));
        assert_eq!(DrawOp::point_pairs(&[a, b, c]), vec![(a, b), (b, c)]);
        assert_eq!(DrawOp::point_pairs(&[a]), vec![(a, a)]);
        assert_eq!(DrawOp::point_pairs(&[]), Vec::new());

        assert_eq!(
            DrawOp::chunk_points(Vec::new()),
            Vec::<Vec<SubCellPos>>::new()
        );
        assert_eq!(DrawOp::chunk_points(line(3)), vec![line(3)]);
        let count = u32::try_from(MAX_POINTS_PER_OP.saturating_mul(3)).unwrap_or(0);
        let pieces = DrawOp::chunk_points(line(count));
        assert!(pieces.iter().all(|piece| piece.len() <= MAX_POINTS_PER_OP));
        assert_eq!(
            pieces.first().and_then(|piece| piece.last()),
            pieces.get(1).and_then(|piece| piece.first())
        );
    }
}
