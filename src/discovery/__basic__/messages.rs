use bevy::prelude::Message;

use crate::identity;

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Create(identity::RoomName),
    Join(identity::RoomName),
    Leave,
    Quit,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum Event {
    LobbyChanged,
    Joined(identity::RoomName),
    RoomReady(identity::RoomName),
    Left(identity::RoomName),
    MembersChanged,
}
