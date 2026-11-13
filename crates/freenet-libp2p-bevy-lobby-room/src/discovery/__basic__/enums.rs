use bevy::prelude::Reflect;
use common_types::Lobby;
use serde::{Deserialize, Serialize};

use crate::discovery;
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LobbyWire {
    HeartbeatTargets(Vec<discovery::state_machine::HeartbeatTarget>),
    Snapshot(Lobby),
}
