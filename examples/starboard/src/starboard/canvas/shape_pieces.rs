use crate::starboard;

const TRIANGLE_VERTICES: [[i64; 2]; 3] = [[0, -1000], [866, 500], [-866, 500]];
const HEXAGON_VERTICES: [[i64; 2]; 6] = [
    [0, -1000],
    [866, -500],
    [866, 500],
    [0, 1000],
    [-866, 500],
    [-866, -500],
];
const STAR_VERTICES: [[i64; 2]; 10] = [
    [0, -1000],
    [225, -309],
    [951, -309],
    [363, 118],
    [588, 809],
    [0, 382],
    [-588, 809],
    [-363, 118],
    [-951, -309],
    [-225, -309],
];

/// The brush shape as convex pieces; a point is inside when it satisfies every half-plane of one
/// piece. Coordinates are in 1/`starboard::canvas::VERTEX_SCALE` of `radius`. The circle has no pieces.
#[must_use]
pub fn shape_pieces(
    shape: starboard::identity::Shape,
    radius: u16,
) -> Vec<Vec<starboard::canvas::HalfPlane>> {
    let r = i64::from(radius.max(1));
    let bound = r.saturating_mul(starboard::canvas::VERTEX_SCALE);
    match shape {
        starboard::identity::Shape::Circle => Vec::new(),
        starboard::identity::Shape::Square => vec![vec![
            starboard::canvas::HalfPlane {
                a: -1,
                b: 0,
                k: bound,
            },
            starboard::canvas::HalfPlane {
                a: 1,
                b: 0,
                k: bound,
            },
            starboard::canvas::HalfPlane {
                a: 0,
                b: -1,
                k: bound,
            },
            starboard::canvas::HalfPlane {
                a: 0,
                b: 1,
                k: bound,
            },
        ]],
        starboard::identity::Shape::Diamond => vec![vec![
            starboard::canvas::HalfPlane {
                a: -1,
                b: -1,
                k: bound,
            },
            starboard::canvas::HalfPlane {
                a: -1,
                b: 1,
                k: bound,
            },
            starboard::canvas::HalfPlane {
                a: 1,
                b: -1,
                k: bound,
            },
            starboard::canvas::HalfPlane {
                a: 1,
                b: 1,
                k: bound,
            },
        ]],
        starboard::identity::Shape::Triangle => {
            starboard::canvas::HalfPlane::origin_sectors(&TRIANGLE_VERTICES, r)
        }
        starboard::identity::Shape::Hexagon => {
            starboard::canvas::HalfPlane::origin_sectors(&HEXAGON_VERTICES, r)
        }
        starboard::identity::Shape::Star => {
            starboard::canvas::HalfPlane::origin_sectors(&STAR_VERTICES, r)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::shape_pieces;
    use crate::starboard::identity::Shape;

    #[test]
    fn test_usage() {
        assert_eq!(shape_pieces(Shape::Circle, 8).len(), 0);
        assert_eq!(shape_pieces(Shape::Square, 8).len(), 1);
        assert_eq!(shape_pieces(Shape::Triangle, 8).len(), 3);
        assert_eq!(shape_pieces(Shape::Star, 8).len(), 10);
    }
}
