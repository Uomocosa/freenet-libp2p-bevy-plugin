use serde::{Deserialize, Serialize};

use crate::net_id;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Hello {
    pub room: Option<net_id::RoomName>,
    pub addrs: Vec<net_id::PeerAddr>,
    pub peers: Vec<net_id::Peer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishTarget {
    pub room: net_id::RoomName,
    pub addrs: Vec<net_id::PeerAddr>,
}
