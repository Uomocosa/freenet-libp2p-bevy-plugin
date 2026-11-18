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
            testing::LobbyTarget::EnterRoomButton(name) => row
                .enter_room_button
                .is_some_and(|button| button.as_str() == name.as_str()),
        })
        .map(|row| testing::LocatedNode {
            centre: *testing::NodeCentre::from_node(row.node, row.transform),
            disabled: row.disabled,
        });
    serde_json::to_value(located).map_err(BrpError::internal)
}
