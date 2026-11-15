use bevy::prelude::Message;
use serde::{Deserialize, Serialize};

use crate::identity;

#[derive(Message, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    Create(identity::ZoneName),
    Join(identity::ZoneName),
    Leave,
    Quit,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum Event {
    LobbyUpdated,
    Joined(identity::ZoneName),
    RoomReady(identity::ZoneName),
    Left(identity::ZoneName),
    RoomMembersUpdated,
    RoomGone(identity::ZoneName),
    HistoryUnavailable(identity::ZoneName),
}
