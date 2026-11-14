use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LobbyTarget {
    RoomNameInput,
    CreateRoomButton,
    Room(String),
}
