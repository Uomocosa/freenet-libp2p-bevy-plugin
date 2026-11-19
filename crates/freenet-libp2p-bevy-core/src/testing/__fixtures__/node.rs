use bevy::math::Vec2;
use bevy::ui::ComputedNode;

#[must_use]
pub fn node(size: Vec2, inverse_scale_factor: f32) -> ComputedNode {
    ComputedNode {
        size,
        inverse_scale_factor,
        ..ComputedNode::default()
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use super::node;

    #[test]
    fn test_usage() {
        let node = node(Vec2::new(100.0, 40.0), 0.5);
        assert_eq!(node.size(), Vec2::new(100.0, 40.0));
        assert_eq!(node.inverse_scale_factor(), 0.5);
    }
}
