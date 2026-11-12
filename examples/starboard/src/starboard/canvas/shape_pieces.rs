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
        starboard::identity::Shape::Triangle => origin_sectors(&TRIANGLE_VERTICES, r),
        starboard::identity::Shape::Hexagon => origin_sectors(&HEXAGON_VERTICES, r),
        starboard::identity::Shape::Star => origin_sectors(&STAR_VERTICES, r),
    }
}

// needed helper: each origin-centred sector of a star-convex polygon is one triangle, and the
// triangle's three edge tests are the three half-planes below
fn origin_sectors(vertices: &[[i64; 2]], radius: i64) -> Vec<Vec<starboard::canvas::HalfPlane>> {
    let len = vertices.len();
    (0..len)
        .filter_map(|idx| {
            let [ax0, ay0] = *vertices.get(idx)?;
            let next = idx.saturating_add(1).checked_rem(len)?;
            let [bx0, by0] = *vertices.get(next)?;
            let (ax, ay) = (ax0.saturating_mul(radius), ay0.saturating_mul(radius));
            let (bx, by) = (bx0.saturating_mul(radius), by0.saturating_mul(radius));
            Some(vec![
                starboard::canvas::HalfPlane {
                    a: ay.saturating_neg(),
                    b: ax,
                    k: 0,
                },
                starboard::canvas::HalfPlane {
                    a: by.saturating_sub(ay).saturating_neg(),
                    b: bx.saturating_sub(ax),
                    k: bx
                        .saturating_sub(ax)
                        .saturating_mul(ay)
                        .saturating_neg()
                        .saturating_add(by.saturating_sub(ay).saturating_mul(ax)),
                },
                starboard::canvas::HalfPlane {
                    a: by,
                    b: bx.saturating_neg(),
                    k: 0,
                },
            ])
        })
        .collect()
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
