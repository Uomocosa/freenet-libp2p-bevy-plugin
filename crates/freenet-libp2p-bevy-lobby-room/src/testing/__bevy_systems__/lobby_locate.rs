use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use serde_json::Value;

use crate::testing;

pub fn lobby_locate(
    In(params): In<Option<Value>>,
    nodes: Query<testing::LocatableNode>,
) -> BrpResult {
    let target: testing::LobbyTarget =
        serde_json::from_value(params.unwrap_or(Value::Null)).map_err(BrpError::internal)?;
    let located = nodes
        .iter()
        .find(
            |(_, _, _, is_name_input, is_create_button, room)| match &target {
                testing::LobbyTarget::RoomNameInput => *is_name_input,
                testing::LobbyTarget::CreateRoomButton => *is_create_button,
                testing::LobbyTarget::Room(name) => {
                    room.is_some_and(|room| room.as_str() == name.as_str())
                }
            },
        )
        .map(
            |(node, transform, disabled, _, _, _)| testing::LocatedNode {
                centre: *testing::NodeCentre::from_node(node, transform),
                disabled,
            },
        );
    serde_json::to_value(located).map_err(BrpError::internal)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::ui::{ComputedNode, InteractionDisabled, UiGlobalTransform};
    use serde_json::Value;
    use serde_json::json;

    use crate::identity;
    use crate::testing;
    use crate::ui;

    fn locate(app: &mut App, target: Value) -> Option<testing::LocatedNode> {
        let system = app.world_mut().register_system(super::lobby_locate);
        let result = app.world_mut().run_system_with(system, Some(target));
        result
            .ok()
            .and_then(Result::ok)
            .and_then(|value| serde_json::from_value(value).ok())
    }

    fn placed_node(x: f32, y: f32) -> (ComputedNode, UiGlobalTransform) {
        (
            ComputedNode {
                inverse_scale_factor: 1.0,
                ..ComputedNode::default()
            },
            UiGlobalTransform::from_xy(x, y),
        )
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.world_mut()
            .spawn((placed_node(200.0, 100.0), ui::discovery::RoomNameInput));
        app.world_mut().spawn((
            placed_node(40.0, 20.0),
            ui::discovery::CreateRoomButton,
            InteractionDisabled,
        ));
        assert_eq!(
            locate(&mut app, json!("RoomNameInput")),
            Some(testing::LocatedNode {
                centre: Vec2::new(200.0, 100.0),
                disabled: false,
            })
        );
        assert_eq!(
            locate(&mut app, json!("CreateRoomButton")),
            Some(testing::LocatedNode {
                centre: Vec2::new(40.0, 20.0),
                disabled: true,
            })
        );
        assert_eq!(locate(&mut app, json!({ "Room": "alpha" })), None);
    }

    #[test]
    fn test_locates_room_button_by_name() {
        let mut app = App::new();
        app.world_mut().spawn((
            placed_node(10.0, 30.0),
            ui::discovery::RoomButton(identity::RoomName::from("alpha")),
        ));
        assert_eq!(
            locate(&mut app, json!({ "Room": "alpha" })),
            Some(testing::LocatedNode {
                centre: Vec2::new(10.0, 30.0),
                disabled: false,
            })
        );
        assert_eq!(locate(&mut app, json!({ "Room": "beta" })), None);
    }
}
