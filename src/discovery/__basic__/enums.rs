use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

use crate::discovery;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum JoinPhase {
    Joining { since: discovery::UnixTime },
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum RoomMembership {
    Occupant,
    HistoryHolder,
    Departed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum HowRoomWasEntered {
    Created,
    Joined,
}
