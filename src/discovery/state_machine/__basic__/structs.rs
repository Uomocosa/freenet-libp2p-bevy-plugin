use serde::{Deserialize, Serialize};

use crate::discovery;
use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Handshake {
    pub room: Option<identity::RoomName>,
    pub addrs: Vec<identity::PeerAddr>,
    pub peers: Vec<identity::Peer>,
    pub display_name: identity::Username,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeartbeatTarget {
    pub room: identity::RoomName,
    pub created_at: discovery::UnixTime,
    pub addrs: Vec<identity::PeerAddr>,
    pub display_name: identity::Username,
    pub membership: discovery::RoomMembership,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeldRoomHistory {
    pub created_at: discovery::UnixTime,
    pub left_at: discovery::UnixTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservedLobbyHeartbeat {
    pub remote_updated_at: discovery::UnixTime,
    pub first_seen_locally_at: discovery::UnixTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub addrs: Vec<identity::PeerAddr>,
    pub seen_at: discovery::UnixTime,
}
