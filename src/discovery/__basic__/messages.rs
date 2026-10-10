use bevy::prelude::Message;

use crate::net_id;

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Create(net_id::RoomName),
    Join(net_id::RoomName),
    Leave,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum Event {
    LobbyChanged,
    Joined(net_id::RoomName),
    Left(net_id::RoomName),
    MembersChanged,
}
