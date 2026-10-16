use std::collections::{BTreeMap, BTreeSet};

use crate::discovery;
use crate::identity;

pub struct State {
    pub me: identity::Peer,
    pub timing: discovery::Timing,
    pub lobby: discovery::Lobby,
    pub room: Option<discovery::JoinedRoom>,
    pub connected: BTreeSet<identity::PeerId>,
    pub candidates: BTreeMap<identity::PeerId, discovery::Heartbeat>,
    pub last_dial: BTreeMap<identity::PeerId, discovery::UnixTime>,
    pub last_handshake: Option<discovery::UnixTime>,
    pub freenet_peers: Vec<identity::FreenetPeer>,
    pub display_name: identity::Username,
}

impl State {
    #[must_use]
    pub const fn new(me: identity::Peer, timing: discovery::Timing) -> Self {
        Self {
            me,
            timing,
            lobby: discovery::Lobby::new(),
            room: None,
            connected: BTreeSet::new(),
            candidates: BTreeMap::new(),
            last_dial: BTreeMap::new(),
            last_handshake: None,
            freenet_peers: Vec::new(),
            display_name: identity::Username(String::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::State;
    use crate::discovery;
    use crate::identity;

    #[test]
    fn test_usage() {
        let state = State::new(
            identity::Peer {
                id: identity::PeerId::from("me"),
                addrs: Vec::new(),
            },
            discovery::Timing::default(),
        );
        assert!(state.room.is_none());
    }
}
