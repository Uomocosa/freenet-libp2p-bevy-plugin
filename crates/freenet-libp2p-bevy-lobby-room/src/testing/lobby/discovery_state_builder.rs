use std::collections::BTreeSet;

use crate::discovery;
use crate::freenet;
use crate::identity;
use discovery::state_machine::DiscoveryState;

pub struct DiscoveryStateBuilder {
    me: identity::PeerId,
    room: Option<discovery::JoinedRoom>,
    lobby: discovery::Lobby,
    connected: BTreeSet<identity::PeerId>,
    freenet_peers: Vec<freenet::FreenetPeer>,
    is_lobby_fetched: bool,
}

impl DiscoveryStateBuilder {
    #[must_use]
    pub fn new(me: identity::PeerId) -> Self {
        Self {
            me,
            room: None,
            lobby: discovery::Lobby::default(),
            connected: BTreeSet::new(),
            freenet_peers: Vec::new(),
            is_lobby_fetched: false,
        }
    }
}

impl DiscoveryStateBuilder {
    #[must_use]
    pub fn in_room(mut self, room: discovery::JoinedRoom) -> Self {
        self.room = Some(room);
        self
    }
    #[must_use]
    pub fn with_lobby(mut self, lobby: discovery::Lobby) -> Self {
        self.lobby = lobby;
        self
    }
    #[must_use]
    pub fn with_connected(mut self, connected: Vec<identity::PeerId>) -> Self {
        self.connected = connected.into_iter().collect();
        self
    }
    #[must_use]
    pub fn with_freenet_peers(mut self, peers: Vec<freenet::FreenetPeer>) -> Self {
        self.freenet_peers = peers;
        self
    }
    #[must_use]
    pub const fn lobby_fetched(mut self) -> Self {
        self.is_lobby_fetched = true;
        self
    }
    #[must_use]
    pub fn build(self) -> DiscoveryState {
        let mut state = DiscoveryState::new(identity::Peer {
            id: self.me,
            addrs: Vec::new(),
        });
        (
            state.room,
            state.lobby,
            state.connected,
            state.freenet_peers,
            state.is_lobby_fetched,
        ) = (
            self.room,
            self.lobby,
            self.connected,
            self.freenet_peers,
            self.is_lobby_fetched,
        );
        state
    }
}

#[cfg(test)]
mod tests {
    use super::DiscoveryStateBuilder;
    use crate::identity;

    #[test]
    fn test_usage() {
        let state = DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .lobby_fetched()
            .build();
        assert!(state.is_lobby_fetched);
        assert_eq!(state.me.id, identity::PeerId::from("me"));
        assert!(state.room.is_none());
    }
}
