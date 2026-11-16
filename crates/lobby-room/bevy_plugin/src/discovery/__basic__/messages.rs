use bevy::prelude::Message;
use serde::{Deserialize, Serialize};

use crate::discovery;

#[derive(Message, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    Create(discovery::RoomName),
    Join(discovery::RoomName),
    Leave,
    Quit,
    Watch(discovery::RoomName),
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum Event {
    LobbyUpdated,
    Joined(discovery::RoomName),
    RoomReady(discovery::RoomName),
    Left(discovery::RoomName),
    RoomMembersUpdated,
    RoomGone(discovery::RoomName),
    HistoryUnavailable(discovery::RoomName),
}
