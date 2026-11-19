use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LobbyTarget {
    RoomNameInput,
    CreateRoomButton,
    Room(String),
    EnterRoomButton(String),
    EscMenuLeaveButton,
    LeaveYesButton,
    LeaveNoButton,
    EscMenuResumeButton,
    EscMenuFriendsButton,
    LeaveDoor,
    FriendAddressInput,
    AddFriendButton,
    FriendsDrawerCloseButton,
}
