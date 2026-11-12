use serde::{Deserialize, Serialize};

use crate::starboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Shape {
    Circle,
    Star,
    Square,
    Triangle,
    Diamond,
    Hexagon,
}

impl Shape {
    pub(crate) fn is_interior(self, radius: u16, dx: i32, dy: i32) -> bool {
        [(1_i32, 0_i32), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .all(|(step_x, step_y)| {
                starboard::canvas::is_inside_shape(
                    self,
                    radius,
                    dx.saturating_add(*step_x),
                    dy.saturating_add(*step_y),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::Shape;

    #[test]
    fn test_usage() {
        assert!(Shape::Square.is_interior(2, 0, 0));
        assert!(!Shape::Square.is_interior(2, 2, 0));
    }
}
