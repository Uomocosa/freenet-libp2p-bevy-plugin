use bevy::prelude::Reflect;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver;

use crate::discovery;
use crate::identity;
use crate::time;

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct JoinedRoom {
    pub name: identity::RoomName,
    pub created_at: time::UnixTime,
    pub phase: discovery::JoinPhase,
    pub members: discovery::Members,
    pub how_entered: discovery::HowRoomWasEntered,
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct LobbyRow {
    pub name: identity::RoomName,
    pub occupants: usize,
    pub room_size: discovery::RoomSize,
    pub occupancy: discovery::RoomOccupancy,
    pub friends: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct RoomRosterEntry {
    pub id: identity::PeerId,
    pub name: String,
    pub status: discovery::MemberRoomStatus,
    pub is_me: bool,
}

pub struct LobbyTaskChannels {
    pub heartbeat_targets: Receiver<Vec<discovery::state_machine::HeartbeatTarget>>,
    pub lobbies: UnboundedSender<discovery::Lobby>,
    pub freenet_peers: UnboundedSender<Vec<identity::FreenetPeer>>,
}
