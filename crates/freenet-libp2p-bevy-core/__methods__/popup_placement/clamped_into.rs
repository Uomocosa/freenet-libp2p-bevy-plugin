use bevy::math::{Rect, Vec2};

#[must_use]
pub fn clamped_into(area: Rect, popup: Vec2, wanted: Vec2) -> Vec2 {
    let low = area.min;
    let high = Vec2::new(
        (area.max.x - popup.x).max(low.x),
        (area.max.y - popup.y).max(low.y),
    );
    wanted.clamp(low, high)
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use super::clamped_into;
    use crate::testing::area;

    #[test]
    fn test_usage() {
        assert_eq!(
            clamped_into(
                area(Vec2::new(960.0, 540.0)),
                Vec2::new(100.0, 40.0),
                Vec2::new(100.0, 100.0)
            ),
            Vec2::new(100.0, 100.0)
        );
    }

    #[test]
    fn clamps_at_the_area_top_and_left() {
        assert_eq!(
            clamped_into(
                area(Vec2::new(960.0, 540.0)),
                Vec2::new(100.0, 40.0),
                Vec2::new(-30.0, -10.0)
            ),
            Vec2::ZERO
        );
    }

    #[test]
    fn clamps_at_the_area_bottom_and_right() {
        assert_eq!(
            clamped_into(
                area(Vec2::new(960.0, 540.0)),
                Vec2::new(100.0, 40.0),
                Vec2::new(2000.0, 2000.0)
            ),
            Vec2::new(860.0, 500.0)
        );
    }

    #[test]
    fn a_popup_larger_than_the_area_pins_to_the_area_min() {
        assert_eq!(
            clamped_into(
                area(Vec2::new(960.0, 540.0)),
                Vec2::new(1000.0, 600.0),
                Vec2::new(10.0, 10.0)
            ),
            Vec2::ZERO
        );
    }

    #[test]
    fn a_popup_inside_the_area_is_not_moved() {
        assert_eq!(
            clamped_into(
                area(Vec2::new(960.0, 540.0)),
                Vec2::new(100.0, 40.0),
                Vec2::new(300.0, 200.0)
            ),
            Vec2::new(300.0, 200.0)
        );
    }
}
