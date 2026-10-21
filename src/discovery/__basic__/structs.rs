use std::collections::BTreeMap;

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::discovery;
use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct RoomMember {
    pub peer: identity::Peer,
    pub display_name: Option<identity::Username>,
    pub updated_at: discovery::UnixTime,
    pub presence: discovery::Presence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct Room {
    pub room_size: discovery::RoomSize,
    pub created_at: discovery::UnixTime,
    pub members: BTreeMap<identity::PeerId, RoomMember>,
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct Member {
    pub room_member: RoomMember,
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
    pub statuses: UnboundedSender<discovery::Status>,
    pub events: UnboundedSender<discovery::Event>,
}
