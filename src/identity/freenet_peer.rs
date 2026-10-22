use std::net::SocketAddr;

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct FreenetPeer {
    pub id: identity::FreenetPeerId,
    pub addr: SocketAddr,
}

impl FreenetPeer {
    #[must_use]
    pub const fn is_ring_ready(peers: &[Self]) -> bool {
        !peers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::FreenetPeer;
    use crate::identity;

    #[test]
    fn test_usage() {
        let peer = FreenetPeer {
            id: identity::FreenetPeerId::from("p"),
            addr: std::net::SocketAddr::from(([127, 0, 0, 1], 1)),
        };
        assert_eq!(peer.id.as_str(), "p");
    }
}
