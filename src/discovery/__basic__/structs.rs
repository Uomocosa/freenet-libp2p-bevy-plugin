use std::collections::BTreeMap;

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::discovery;
use crate::net_id;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct Presence {
    pub addrs: Vec<net_id::PeerAddr>,
    pub updated_at: discovery::UnixTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct RoomEntry {
    pub capacity: u16,
    pub members: BTreeMap<net_id::PeerId, Presence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct Member {
    pub presence: Presence,
    pub status: discovery::LinkStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct Room {
    pub name: net_id::RoomName,
    pub members: discovery::Members,
}

pub struct Channels {
    pub net: discovery::libp2p::Link,
    pub commands: UnboundedReceiver<discovery::Command>,
    pub snapshots: UnboundedSender<discovery::Snapshot>,
    pub events: UnboundedSender<discovery::Event>,
}
