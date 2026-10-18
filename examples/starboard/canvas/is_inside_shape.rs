use crate::starboard;

pub fn is_inside_shape(shape: starboard::identity::Shape, radius: u16, dx: i32, dy: i32) -> bool {
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

    let r = i64::from(radius.max(1));
    let x = i64::from(dx);
    let y = i64::from(dy);
    if x.abs() > r || y.abs() > r {
        return false;
    }
    if x == 0 && y == 0 {
        return true;
    }
    match shape {
        starboard::identity::Shape::Circle => {
            x.saturating_mul(x).saturating_add(y.saturating_mul(y)) <= r.saturating_mul(r)
        }
        starboard::identity::Shape::Square => x.abs().max(y.abs()) <= r,
        starboard::identity::Shape::Diamond => x.abs().saturating_add(y.abs()) <= r,
        starboard::identity::Shape::Triangle => is_in_star_polygon(&TRIANGLE_VERTICES, r, x, y),
        starboard::identity::Shape::Hexagon => is_in_star_polygon(&HEXAGON_VERTICES, r, x, y),
        starboard::identity::Shape::Star => is_in_star_polygon(&STAR_VERTICES, r, x, y),
    }
}

// needed helper: test if (x, y) lies in any origin-centered sector triangle of a star-convex polygon
fn is_in_star_polygon(vertices: &[[i64; 2]], radius: i64, x: i64, y: i64) -> bool {
    const VERTEX_SCALE: i64 = 1000;
    let px = x.saturating_mul(VERTEX_SCALE);
    let py = y.saturating_mul(VERTEX_SCALE);
    let len = vertices.len();
    for idx in 0..len {
        let Some(&[ax0, ay0]) = vertices.get(idx) else {
            continue;
        };
        let next_idx = idx.saturating_add(1).checked_rem(len).unwrap_or(0);
        let Some(&[bx0, by0]) = vertices.get(next_idx) else {
            continue;
        };
        let ax = ax0.saturating_mul(radius);
        let ay = ay0.saturating_mul(radius);
        let bx = bx0.saturating_mul(radius);
        let by = by0.saturating_mul(radius);
        let c1 = ax.saturating_mul(py).saturating_sub(ay.saturating_mul(px));
        let c2 = (bx.saturating_sub(ax))
            .saturating_mul(py.saturating_sub(ay))
            .saturating_sub((by.saturating_sub(ay)).saturating_mul(px.saturating_sub(ax)));
        let c3 = by
            .saturating_mul(px.saturating_sub(bx))
            .saturating_sub(bx.saturating_mul(py.saturating_sub(by)));
        if c1 >= 0 && c2 >= 0 && c3 >= 0 {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::is_inside_shape;
    use crate::starboard::identity::{SHAPES, Shape};

    fn render_ascii(shape: Shape, radius: u16) -> String {
        let r = i32::from(radius);
        let mut lines = Vec::new();
        for dy in -r..=r {
            let mut row = String::new();
            for dx in -r..=r {
                if is_inside_shape(shape, radius, dx, dy) {
                    row.push('#');
                } else {
                    row.push('.');
                }
            }
            lines.push(row);
        }
        lines.join("\n")
    }

    #[test]
    fn test_usage() {
        for shape in SHAPES {
            assert!(is_inside_shape(shape, 1, 0, 0));
            assert!(is_inside_shape(shape, 16, 0, 0));
            assert!(!is_inside_shape(shape, 16, 17, 0));
            assert!(!is_inside_shape(shape, 16, 0, -17));
        }

        assert!(is_inside_shape(Shape::Square, 8, 8, 8));
        assert!(!is_inside_shape(Shape::Circle, 8, 8, 8));
        assert!(!is_inside_shape(Shape::Diamond, 8, 5, 5));
        assert!(is_inside_shape(Shape::Diamond, 8, 4, 4));
        assert!(is_inside_shape(Shape::Triangle, 10, 0, -9));
        assert!(!is_inside_shape(Shape::Triangle, 10, 0, 6));
        assert!(is_inside_shape(Shape::Hexagon, 10, 0, -9));
        assert!(is_inside_shape(Shape::Hexagon, 10, 0, 9));
        assert!(!is_inside_shape(Shape::Hexagon, 10, 9, 0));
        assert!(is_inside_shape(Shape::Star, 10, 0, -9));
        assert!(
            !is_inside_shape(Shape::Star, 10, 0, -6) || !is_inside_shape(Shape::Star, 10, 5, -5)
        );
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
            render_ascii(Shape::Circle, 2),
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
