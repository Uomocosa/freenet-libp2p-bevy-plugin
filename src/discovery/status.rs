use std::collections::BTreeSet;
use std::time::Duration;

use bevy::prelude::Resource;

use crate::discovery;
use crate::identity;

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub lobby: discovery::Lobby,
    pub room: Option<discovery::JoinedRoom>,
    pub connected: BTreeSet<identity::PeerId>,
    pub freenet_peers: Vec<identity::FreenetPeer>,
    pub is_lobby_attached: bool,
    pub self_peer_id: Option<identity::PeerId>,
    pub lease_ttl: Duration,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            lobby: discovery::Lobby::default(),
            room: None,
            connected: BTreeSet::new(),
            freenet_peers: Vec::new(),
            is_lobby_attached: false,
            self_peer_id: None,
            lease_ttl: discovery::HolderHeartbeatIntervals::default().ttl,
        }
    }
}

#[rustfmt::skip]
impl Status {
    #[must_use]
    pub fn link_status(&self, peer: &identity::PeerId) -> discovery::LinkStatus {
        if self.connected.contains(peer) {
            discovery::LinkStatus::Connected
        } else {
            discovery::LinkStatus::Lost
        }
    }

    pub fn connected_members(&self) -> impl Iterator<Item = &identity::PeerId> {
        let room = self.room.as_ref();
        self.connected
            .iter()
            .filter(move |peer| room.is_some_and(|room| room.members.contains_key(*peer)))
    }
}

// no test_usage necessary
