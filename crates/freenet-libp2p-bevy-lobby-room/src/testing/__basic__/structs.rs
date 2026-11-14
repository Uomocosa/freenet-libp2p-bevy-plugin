use bevy::ecs::query::{Has, QueryData};
use bevy::math::Vec2;
use bevy::ui::{ComputedNode, InteractionDisabled, UiGlobalTransform};
use serde::{Deserialize, Serialize};

use crate::discovery;
use crate::ui;

pub struct LobbyStepView<'a> {
    pub status: &'a discovery::state_machine::DiscoveryState,
    pub join_error: Option<&'a str>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LobbyStatus {
    pub rooms: Vec<String>,
    pub joined_room: Option<String>,
    pub other_members: usize,
    pub connected: usize,
    pub lobby_fetched: bool,
    pub join_error: Option<String>,
}

#[derive(QueryData)]
pub struct LocatableNode {
    pub node: &'static ComputedNode,
    pub transform: &'static UiGlobalTransform,
    pub disabled: Has<InteractionDisabled>,
    pub room_name_input: Has<ui::discovery::RoomNameInput>,
    pub create_room_button: Has<ui::discovery::CreateRoomButton>,
    pub esc_menu_leave_button: Has<ui::discovery::EscMenuLeaveButton>,
    pub leave_yes_button: Has<ui::discovery::LeaveYesButton>,
    pub leave_no_button: Has<ui::discovery::LeaveNoButton>,
    pub esc_menu_resume_button: Has<ui::discovery::EscMenuResumeButton>,
    pub esc_menu_friends_button: Has<ui::discovery::EscMenuFriendsButton>,
    pub leave_door: Has<ui::discovery::LeaveDoor>,
    pub friend_address_input: Has<ui::direct_dial::FriendAddressInput>,
    pub add_friend_button: Has<ui::direct_dial::AddFriendButton>,
    pub friends_drawer_close_button: Has<ui::direct_dial::FriendsDrawerCloseButton>,
    pub room: Option<&'static ui::discovery::RoomButton>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LocatedNode {
    pub centre: Vec2,
    pub disabled: bool,
}

// no test_usage necessary
