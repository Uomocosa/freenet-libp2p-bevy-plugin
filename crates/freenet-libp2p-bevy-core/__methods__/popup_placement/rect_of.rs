use bevy::math::{Rect, Vec2};
use bevy::ui::{ComputedNode, UiGlobalTransform};
#[must_use]
pub fn rect_of(node: &ComputedNode, transform: &UiGlobalTransform) -> Rect {
    let inverse = node.inverse_scale_factor();
    let size = node.size();
    let logical_size = Vec2::new(size.x * inverse, size.y * inverse);
    let (_, _, centre) = transform.to_scale_angle_translation();
    let logical_centre = Vec2::new(centre.x * inverse, centre.y * inverse);
    Rect::from_center_size(logical_centre, logical_size)
}

#[cfg(test)]
mod tests {
    use bevy::math::{Rect, Vec2};
    use bevy::ui::UiGlobalTransform;

    use super::rect_of;
    use crate::testing::node;

    #[test]
    fn test_usage() {
        assert_eq!(
            rect_of(
                &node(Vec2::new(100.0, 40.0), 1.0),
                &UiGlobalTransform::from_xy(200.0, 100.0)
            ),
            Rect::from_center_size(Vec2::new(200.0, 100.0), Vec2::new(100.0, 40.0))
        );
    }

    #[test]
    fn rect_of_centres_on_the_translation_and_scales() {
        assert_eq!(
            rect_of(
                &node(Vec2::new(100.0, 40.0), 0.5),
                &UiGlobalTransform::from_xy(200.0, 100.0)
            ),
            Rect::from_center_size(Vec2::new(100.0, 50.0), Vec2::new(50.0, 20.0))
        );
    }
}
