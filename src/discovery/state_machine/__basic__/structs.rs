use serde::{Deserialize, Serialize};

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
    pub addrs: Vec<identity::PeerAddr>,
    pub display_name: identity::Username,
}
