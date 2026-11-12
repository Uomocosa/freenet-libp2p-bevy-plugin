use crate::starboard;

/// The consecutive pairs of a polyline; a single point is paired with itself, so a dab rasterizes.
#[must_use]
pub fn point_pairs(
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

#[cfg(test)]
mod tests {
    use super::point_pairs;
    use crate::starboard::canvas::SubCellPos;

    #[test]
    fn test_usage() {
        let (a, b, c) = (
            SubCellPos { x: 1, y: 1 },
            SubCellPos { x: 2, y: 1 },
            SubCellPos { x: 3, y: 1 },
        );
        assert_eq!(point_pairs(&[a, b, c]), vec![(a, b), (b, c)]);
        assert_eq!(point_pairs(&[a]), vec![(a, a)]);
        assert_eq!(point_pairs(&[]), Vec::new());
    }
}
