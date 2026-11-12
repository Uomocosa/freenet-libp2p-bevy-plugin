use crate::starboard;

/// The polyline with every point removed that lies strictly inside the straight run between its
/// kept neighbours, and every repeat of the previous point. The brush swept along the kept points
/// covers exactly the cells swept along the original points, so the painted cells do not change.
#[must_use]
pub fn drop_collinear_points(
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
            let is_straight = first.0.saturating_mul(second.1) == first.1.saturating_mul(second.0);
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

#[cfg(test)]
mod tests {
    use super::drop_collinear_points;
    use crate::starboard::canvas::SubCellPos;

    fn at(x: u32, y: u32) -> SubCellPos {
        SubCellPos { x, y }
    }

    #[test]
    fn test_usage() {
        assert_eq!(
            drop_collinear_points(&[at(0, 0), at(4, 0), at(8, 0), at(8, 5)]),
            vec![at(0, 0), at(8, 0), at(8, 5)]
        );
        assert_eq!(
            drop_collinear_points(&[at(0, 0), at(4, 0), at(2, 0)]),
            vec![at(0, 0), at(4, 0), at(2, 0)]
        );
        assert_eq!(drop_collinear_points(&[at(1, 1), at(1, 1)]), vec![at(1, 1)]);
        assert_eq!(drop_collinear_points(&[]), Vec::new());
    }
}
