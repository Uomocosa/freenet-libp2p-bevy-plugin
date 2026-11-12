use crate::starboard;

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
        x.saturating_mul(starboard::canvas::VERTEX_SCALE),
        y.saturating_mul(starboard::canvas::VERTEX_SCALE),
    );
    starboard::canvas::shape_pieces(shape, radius)
        .iter()
        .any(|piece| {
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
