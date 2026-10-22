use bevy::prelude::Reflect;
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::discovery;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum LinkStatus {
    Connected,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum JoinPhase {
    Joining { since: discovery::UnixTime },
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum FreenetRequestKind {
    #[display("lobby fetch")]
    FetchLobby,
    #[display("lobby deploy")]
    DeployLobby,
    #[display("peer query")]
    QueryPeers,
    #[display("lobby poll")]
    PollLobby,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum RoomMembership {
    Occupant,
    HistoryHolder,
    Departed,
}
