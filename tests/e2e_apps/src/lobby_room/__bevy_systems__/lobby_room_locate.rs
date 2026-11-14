use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use bevy::ui::{ComputedNode, InteractionDisabled, UiGlobalTransform};
use freenet_libp2p_bevy_plugin::lobby_room::testing::LocatedNode;
use freenet_libp2p_bevy_plugin::testing::NodeCentre;
use lobby_room::ui::{JoinButton, NameField, NewRoomButton};
use serde_json::Value;

use crate::lobby_room::LobbyRoomTarget;

pub fn lobby_room_locate(
    In(params): In<Option<Value>>,
    name_fields: Query<(&ComputedNode, &UiGlobalTransform), With<NameField>>,
    new_room_buttons: Query<
        (&ComputedNode, &UiGlobalTransform, Has<InteractionDisabled>),
        With<NewRoomButton>,
    >,
    joins: Query<(&ComputedNode, &UiGlobalTransform, &JoinButton)>,
) -> BrpResult {
    let target: LobbyRoomTarget =
        serde_json::from_value(params.unwrap_or(Value::Null)).map_err(BrpError::internal)?;
    let located_at =
        |node: &ComputedNode, transform: &UiGlobalTransform, disabled: bool| LocatedNode {
            centre: *NodeCentre::from_node(node, transform),
            disabled,
        };
    let located = match target {
        LobbyRoomTarget::NameField => name_fields
            .iter()
            .next()
            .map(|(node, transform)| located_at(node, transform, false)),
        LobbyRoomTarget::NewRoomButton => new_room_buttons
            .iter()
            .next()
            .map(|(node, transform, disabled)| located_at(node, transform, disabled)),
        LobbyRoomTarget::Join(name) => joins
            .iter()
            .find(|(_, _, join)| join.0.as_str() == name)
            .map(|(node, transform, _)| located_at(node, transform, false)),
    };
    serde_json::to_value(located).map_err(BrpError::internal)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::ui::{ComputedNode, UiGlobalTransform};
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::testing::LocatedNode;
    use lobby_room::ui::{JoinButton, NameField};
    use serde_json::{Value, json};

    fn placed(x: f32, y: f32) -> (ComputedNode, UiGlobalTransform) {
        (
            ComputedNode {
                inverse_scale_factor: 1.0,
                ..ComputedNode::default()
            },
            UiGlobalTransform::from_xy(x, y),
        )
    }

    fn locate(app: &mut App, target: Value) -> Option<LocatedNode> {
        let system = app.world_mut().register_system(super::lobby_room_locate);
        let result = app.world_mut().run_system_with(system, Some(target));
        result
            .ok()
            .and_then(Result::ok)
            .and_then(|value| serde_json::from_value(value).ok())
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.world_mut().spawn((placed(200.0, 100.0), NameField));
        app.world_mut().spawn((
            placed(300.0, 50.0),
            JoinButton(identity::RoomName::from("alpha")),
        ));
        assert_eq!(
            locate(&mut app, json!("NameField")).map(|node| node.centre),
            Some(Vec2::new(200.0, 100.0))
        );
        assert_eq!(
            locate(&mut app, json!({ "Join": "alpha" })).map(|node| node.centre),
            Some(Vec2::new(300.0, 50.0))
        );
        assert_eq!(locate(&mut app, json!("NewRoomButton")), None);
    }
}
