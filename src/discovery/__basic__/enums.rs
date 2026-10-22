use bevy::prelude::Reflect;
use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum LinkStatus {
    Connected,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
pub enum RoomPhase {
    #[default]
    Joining,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum LobbyRequestKind {
    #[display("lobby fetch")]
    FetchLobby,
    #[display("lobby deploy")]
    DeployLobby,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum RoomMembership {
    InRoom,
    HoldingHistory,
    Quit,
}
