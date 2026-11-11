use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

use crate::time;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum JoinPhase {
    Joining { since: time::UnixTime },
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum HowRoomWasEntered {
    Created,
    Joined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect)]
pub enum RoomOccupancy {
    Empty,
    HasSpace,
    Full,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Reflect)]
pub enum OwnRoomStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect)]
pub enum MemberRoomStatus {
    Connecting,
    Connected,
}
