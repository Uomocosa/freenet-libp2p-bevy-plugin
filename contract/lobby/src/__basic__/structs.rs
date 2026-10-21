use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::UnixTime;
use crate::basic::enums::Presence;
use crate::basic::newtypes::PeerId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Peer {
    pub id: PeerId,
    pub addrs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomMember {
    pub peer: Peer,
    pub display_name: Option<String>,
    pub updated_at: UnixTime,
    pub presence: Presence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Room {
    pub room_size: u16,
    pub created_at: UnixTime,
    pub members: BTreeMap<PeerId, RoomMember>,
}
