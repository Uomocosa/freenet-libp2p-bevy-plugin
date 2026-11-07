use std::collections::HashSet;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use crate::identity;
use crate::p2p;

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize, Deref, DerefMut)]
pub struct ConnectedPeers(pub HashSet<identity::PeerId>);

#[atomic_delegates]
impl ConnectedPeers {
    pub fn apply_connection_event(&mut self, event: &p2p::Libp2pEvent) -> bool {}
}

#[cfg(test)]
mod tests {
    use super::ConnectedPeers;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let mut peers = ConnectedPeers::default();
        assert!(peers.is_empty());
        assert!(
            peers.apply_connection_event(&p2p::Libp2pEvent::PeerConnected(identity::PeerId::from(
                "alice"
            )))
        );
        assert!(peers.contains(&identity::PeerId::from("alice")));
        assert!(
            !peers.apply_connection_event(&p2p::Libp2pEvent::PeerConnected(
                identity::PeerId::from("alice")
            ))
        );
        assert!(
            peers.apply_connection_event(&p2p::Libp2pEvent::PeerDisconnected(
                identity::PeerId::from("alice")
            ))
        );
        assert!(peers.is_empty());
    }
}
