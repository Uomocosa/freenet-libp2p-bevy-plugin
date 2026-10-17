use std::collections::{BTreeMap, BTreeSet};

use crate::discovery;
use crate::identity;

pub struct State {
    pub me: identity::Peer,
    pub lobby: discovery::Lobby,
    pub room: Option<discovery::JoinedRoom>,
    pub connected: BTreeSet<identity::PeerId>,
    pub candidates: BTreeMap<identity::PeerId, discovery::state_machine::Candidate>,
    pub last_dial: BTreeMap<identity::PeerId, discovery::UnixTime>,
    pub last_handshake: Option<discovery::UnixTime>,
    pub joining_since: Option<discovery::UnixTime>,
    pub freenet_peers: Vec<identity::FreenetPeer>,
    pub is_lobby_attached: bool,
    pub display_name: identity::Username,
}

impl State {
    #[must_use]
    pub const fn new(me: identity::Peer) -> Self {
        Self {
            me,
            lobby: discovery::Lobby(BTreeMap::new()),
            room: None,
            connected: BTreeSet::new(),
            candidates: BTreeMap::new(),
            last_dial: BTreeMap::new(),
            last_handshake: None,
            joining_since: None,
            freenet_peers: Vec::new(),
            is_lobby_attached: false,
            display_name: identity::Username(String::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::State;
    use crate::identity;

    #[test]
    fn test_usage() {
        let state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        assert!(state.room.is_none());
    }
}
