use serde::{Deserialize, Serialize};

use crate::net_id;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Peer {
    pub id: net_id::PeerId,
    pub addrs: Vec<net_id::PeerAddr>,
}
