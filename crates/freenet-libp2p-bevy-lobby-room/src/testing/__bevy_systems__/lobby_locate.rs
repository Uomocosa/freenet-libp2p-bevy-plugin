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
        .find(|row| match &target {
            testing::LobbyTarget::RoomNameInput => row.room_name_input,
            testing::LobbyTarget::CreateRoomButton => row.create_room_button,
            testing::LobbyTarget::EscMenuLeaveButton => row.esc_menu_leave_button,
            testing::LobbyTarget::LeaveYesButton => row.leave_yes_button,
            testing::LobbyTarget::LeaveNoButton => row.leave_no_button,
            testing::LobbyTarget::EscMenuResumeButton => row.esc_menu_resume_button,
            testing::LobbyTarget::EscMenuFriendsButton => row.esc_menu_friends_button,
            testing::LobbyTarget::LeaveDoor => row.leave_door,
            testing::LobbyTarget::FriendAddressInput => row.friend_address_input,
            testing::LobbyTarget::AddFriendButton => row.add_friend_button,
            testing::LobbyTarget::FriendsDrawerCloseButton => row.friends_drawer_close_button,
            testing::LobbyTarget::Room(name) => {
                row.room.is_some_and(|room| room.as_str() == name.as_str())
            }
        })
        .map(|row| testing::LocatedNode {
            centre: *testing::NodeCentre::from_node(row.node, row.transform),
            disabled: row.disabled,
        });
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

    #[test]
    fn test_locates_esc_menu_and_friends_widgets() {
        let mut app = App::new();
        app.world_mut()
            .spawn((placed_node(1.0, 1.0), ui::discovery::EscMenuLeaveButton));
        app.world_mut()
            .spawn((placed_node(2.0, 2.0), ui::discovery::LeaveYesButton));
        app.world_mut()
            .spawn((placed_node(3.0, 3.0), ui::discovery::LeaveNoButton));
        app.world_mut()
            .spawn((placed_node(4.0, 4.0), ui::discovery::EscMenuResumeButton));
        app.world_mut()
            .spawn((placed_node(5.0, 5.0), ui::discovery::EscMenuFriendsButton));
        app.world_mut()
            .spawn((placed_node(6.0, 6.0), ui::discovery::LeaveDoor));
        app.world_mut()
            .spawn((placed_node(7.0, 7.0), ui::direct_dial::FriendAddressInput));
        app.world_mut()
            .spawn((placed_node(8.0, 8.0), ui::direct_dial::AddFriendButton));
        app.world_mut().spawn((
            placed_node(9.0, 9.0),
            ui::direct_dial::FriendsDrawerCloseButton,
        ));
        for (name, centre) in [
            ("EscMenuLeaveButton", 1.0),
            ("LeaveYesButton", 2.0),
            ("LeaveNoButton", 3.0),
            ("EscMenuResumeButton", 4.0),
            ("EscMenuFriendsButton", 5.0),
            ("LeaveDoor", 6.0),
            ("FriendAddressInput", 7.0),
            ("AddFriendButton", 8.0),
            ("FriendsDrawerCloseButton", 9.0),
        ] {
            assert_eq!(
                locate(&mut app, json!(name)),
                Some(testing::LocatedNode {
                    centre: Vec2::new(centre, centre),
                    disabled: false,
                }),
                "{name} locates its own node"
            );
        }
    }
}
