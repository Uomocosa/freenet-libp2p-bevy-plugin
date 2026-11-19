use bevy::math::Vec2;
use bevy::ui::{ComputedNode, UiGlobalTransform};
use derive_more::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Deref)]
pub struct NodeCentre(pub Vec2);

impl NodeCentre {
    #[must_use]
    pub fn from_node(node: &ComputedNode, transform: &UiGlobalTransform) -> Self {
        let (_, _, translation) = transform.to_scale_angle_translation();
        let inverse_scale = node.inverse_scale_factor();
        Self(Vec2::new(
            translation.x * inverse_scale,
            translation.y * inverse_scale,
        ))
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;
    use bevy::ui::{ComputedNode, UiGlobalTransform};

    use super::NodeCentre;

    #[test]
    fn test_usage() {
        let node = ComputedNode {
            inverse_scale_factor: 1.0,
            ..ComputedNode::default()
        };
        let transform = UiGlobalTransform::from_xy(200.0, 100.0);
        assert_eq!(
            *NodeCentre::from_node(&node, &transform),
            Vec2::new(200.0, 100.0)
        );
    }

    #[test]
    fn test_centre_at_scale_two() {
        let node = ComputedNode {
            inverse_scale_factor: 0.5,
            ..ComputedNode::default()
        };
        let transform = UiGlobalTransform::from_xy(200.0, 100.0);
        assert_eq!(
            *NodeCentre::from_node(&node, &transform),
            Vec2::new(100.0, 50.0)
        );
    }
}
