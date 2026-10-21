use std::time::Duration;

use bevy::prelude::Resource;

use crate::discovery;
use crate::identity;

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub lobby: discovery::Lobby,
    pub room: Option<discovery::JoinedRoom>,
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
            freenet_peers: Vec::new(),
            is_lobby_attached: false,
            self_peer_id: None,
            lease_ttl: discovery::HolderHeartbeatIntervals::default().ttl,
        }
    }
}

// no test_usage necessary
