use std::collections::BTreeMap;

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::discovery;
use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct Heartbeat {
    pub addrs: Vec<identity::PeerAddr>,
    pub updated_at: discovery::UnixTime,
    pub display_name: Option<identity::Username>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct Room {
    pub room_size: discovery::RoomSize,
    pub created_at: discovery::UnixTime,
    pub members: BTreeMap<identity::PeerId, Heartbeat>,
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct Member {
    pub heartbeat: Heartbeat,
    pub status: discovery::LinkStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct JoinedRoom {
    pub name: identity::RoomName,
    pub created_at: discovery::UnixTime,
    pub phase: discovery::RoomPhase,
    pub members: discovery::Members,
}

pub struct RunChannels {
    pub net: discovery::libp2p::NetLink,
    pub commands: UnboundedReceiver<discovery::Command>,
    pub snapshots: UnboundedSender<discovery::Snapshot>,
    pub events: UnboundedSender<discovery::Event>,
}
