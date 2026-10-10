use std::collections::{BTreeMap, BTreeSet};

use crate::discovery;
use crate::net_id;

pub struct State {
    pub me: net_id::Peer,
    pub timing: discovery::Timing,
    pub lobby: discovery::Lobby,
    pub room: Option<discovery::Room>,
    pub connected: BTreeSet<net_id::PeerId>,
    pub candidates: BTreeMap<net_id::PeerId, discovery::Presence>,
    pub last_dial: BTreeMap<net_id::PeerId, discovery::UnixTime>,
    pub last_hello: Option<discovery::UnixTime>,
}

impl State {
    #[must_use]
    pub const fn new(me: net_id::Peer, timing: discovery::Timing) -> Self {
        Self {
            me,
            timing,
            lobby: discovery::Lobby::new(),
            room: None,
            connected: BTreeSet::new(),
            candidates: BTreeMap::new(),
            last_dial: BTreeMap::new(),
            last_hello: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::State;
    use crate::discovery;
    use crate::net_id;

    #[test]
    fn test_usage() {
        let state = State::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: Vec::new(),
            },
            discovery::Timing::default(),
        );
        assert!(state.room.is_none());
    }
}
