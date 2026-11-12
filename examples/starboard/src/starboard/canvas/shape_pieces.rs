use crate::starboard;
use crate::starboard::canvas::HalfPlane;

const VERTEX_SCALE: i64 = 1000;
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

/// Whether the brush shape, centred at the origin, covers the offset `(dx, dy)`; the same units as
/// `shape_pieces`.
#[must_use]
pub fn is_inside_shape(shape: starboard::identity::Shape, radius: u16, dx: i32, dy: i32) -> bool {
    let r = i64::from(radius.max(1));
    let x = i64::from(dx);
    let y = i64::from(dy);
    if x.abs() > r || y.abs() > r {
        return false;
    }
    if x == 0 && y == 0 {
        return true;
    }
    if shape == starboard::identity::Shape::Circle {
        return x.saturating_mul(x).saturating_add(y.saturating_mul(y)) <= r.saturating_mul(r);
    }
    let (px, py) = (
        x.saturating_mul(VERTEX_SCALE),
        y.saturating_mul(VERTEX_SCALE),
    );
    shape_pieces(shape, radius).iter().any(|piece| {
        piece.iter().all(|half_plane| {
            half_plane
                .a
                .saturating_mul(px)
                .saturating_add(half_plane.b.saturating_mul(py))
                .saturating_add(half_plane.k)
                >= 0
        })
    })
}

/// The brush shape as convex pieces; a point is inside when it satisfies every half-plane of one
/// piece. Coordinates are in 1/`VERTEX_SCALE` of `radius`. The circle has no pieces.
#[must_use]
pub fn shape_pieces(shape: starboard::identity::Shape, radius: u16) -> Vec<Vec<HalfPlane>> {
    let r = i64::from(radius.max(1));
    let bound = r.saturating_mul(VERTEX_SCALE);
    match shape {
        starboard::identity::Shape::Circle => Vec::new(),
        starboard::identity::Shape::Square => vec![vec![
            HalfPlane {
                a: -1,
                b: 0,
                k: bound,
            },
            HalfPlane {
                a: 1,
                b: 0,
                k: bound,
            },
            HalfPlane {
                a: 0,
                b: -1,
                k: bound,
            },
            HalfPlane {
                a: 0,
                b: 1,
                k: bound,
            },
        ]],
        starboard::identity::Shape::Diamond => vec![vec![
            HalfPlane {
                a: -1,
                b: -1,
                k: bound,
            },
            HalfPlane {
                a: -1,
                b: 1,
                k: bound,
            },
            HalfPlane {
                a: 1,
                b: -1,
                k: bound,
            },
            HalfPlane {
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
fn origin_sectors(vertices: &[[i64; 2]], radius: i64) -> Vec<Vec<HalfPlane>> {
    let len = vertices.len();
    (0..len)
        .filter_map(|idx| {
            let [ax0, ay0] = *vertices.get(idx)?;
            let next = idx.saturating_add(1).checked_rem(len)?;
            let [bx0, by0] = *vertices.get(next)?;
            let (ax, ay) = (ax0.saturating_mul(radius), ay0.saturating_mul(radius));
            let (bx, by) = (bx0.saturating_mul(radius), by0.saturating_mul(radius));
            Some(vec![
                HalfPlane {
                    a: ay.saturating_neg(),
                    b: ax,
                    k: 0,
                },
                HalfPlane {
                    a: by.saturating_sub(ay).saturating_neg(),
                    b: bx.saturating_sub(ax),
                    k: bx
                        .saturating_sub(ax)
                        .saturating_mul(ay)
                        .saturating_neg()
                        .saturating_add(by.saturating_sub(ay).saturating_mul(ax)),
                },
                HalfPlane {
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
    use super::is_inside_shape as is_inside;
    use crate::starboard::identity::{SHAPES, Shape};

    fn render_ascii(shape: Shape, radius: u16) -> String {
        let r = i32::from(radius);
        let mut lines = Vec::new();
        for dy in r.saturating_neg()..=r {
            let mut row = String::new();
            for dx in r.saturating_neg()..=r {
                row.push(if is_inside(shape, radius, dx, dy) {
                    '#'
                } else {
                    '.'
                });
            }
            lines.push(row);
        }
        lines.join("\n")
    }

    #[test]
    fn test_usage() {
        for shape in SHAPES {
            assert!(is_inside(shape, 1, 0, 0));
            assert!(!is_inside(shape, 16, 17, 0));
            assert!(!is_inside(shape, 16, 0, -17));
        }
        assert!(is_inside(Shape::Square, 8, 8, 8));
        assert!(!is_inside(Shape::Circle, 8, 8, 8));
        assert!(!is_inside(Shape::Diamond, 8, 5, 5));
        assert!(is_inside(Shape::Triangle, 10, 0, -9));
        assert!(!is_inside(Shape::Triangle, 10, 0, 6));
        assert!(is_inside(Shape::Hexagon, 10, 0, 9));
        assert!(!is_inside(Shape::Hexagon, 10, 9, 0));
        assert!(is_inside(Shape::Star, 10, 0, -9));
    }

    #[test]
    fn small_rasters_match_expected_silhouettes() {
        assert_eq!(
            render_ascii(Shape::Square, 2),
            "#####\n#####\n#####\n#####\n#####"
        );
        assert_eq!(
            render_ascii(Shape::Diamond, 2),
            "..#..\n.###.\n#####\n.###.\n..#.."
        );
        assert_eq!(
            render_ascii(Shape::Triangle, 4),
            "....#....\n....#....\n...###...\n...###...\n..#####..\n..#####..\n.#######.\n.........\n........."
        );
        assert_eq!(
            render_ascii(Shape::Hexagon, 4),
            "....#....\n...###...\n.#######.\n.#######.\n.#######.\n.#######.\n.#######.\n...###...\n....#...."
        );
        assert_eq!(
            render_ascii(Shape::Star, 4),
            "....#....\n....#....\n....#....\n.#######.\n..#####..\n...###...\n...#.#...\n.........\n........."
        );
    }
}
