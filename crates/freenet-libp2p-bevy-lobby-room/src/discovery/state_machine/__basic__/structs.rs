use serde::{Deserialize, Serialize};

use crate::discovery;
use crate::identity;
use crate::time;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Handshake {
    pub room: Option<identity::RoomName>,
    pub addrs: Vec<identity::PeerAddr>,
    pub peers: Vec<identity::Peer>,
    pub display_name: identity::Username,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatTarget {
    pub room: identity::RoomName,
    pub created_at: time::UnixTime,
    pub addrs: Vec<identity::PeerAddr>,
    pub display_name: identity::Username,
    pub membership: discovery::RoomMembership,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeldRoomHistory {
    pub created_at: time::UnixTime,
    pub left_at: time::UnixTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservedLobbyHeartbeat {
    pub remote_updated_at: time::UnixTime,
    pub first_seen_locally_at: time::UnixTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub addrs: Vec<identity::PeerAddr>,
    pub seen_at: time::UnixTime,
}
