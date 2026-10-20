use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use crate::discovery;
use crate::identity;

pub struct State {
    pub me: identity::Peer,
    pub lobby: discovery::Lobby,
    pub room: Option<discovery::JoinedRoom>,
    pub connected: BTreeSet<identity::PeerId>,
    pub candidates: BTreeMap<identity::PeerId, discovery::state_machine::Candidate>,
    pub last_dialed_at: BTreeMap<identity::PeerId, discovery::UnixTime>,
    pub last_handshake_at: Option<discovery::UnixTime>,
    pub joining_since: Option<discovery::UnixTime>,
    pub freenet_peers: Vec<identity::FreenetPeer>,
    pub is_lobby_attached: bool,
    pub display_name: identity::Username,
    pub game: identity::GameId,
    pub held_rooms: BTreeMap<identity::RoomName, discovery::state_machine::HeldRoomHistory>,
    pub evicted_rooms: BTreeMap<identity::RoomName, discovery::UnixTime>,
    pub history_size_limits: discovery::HistorySizeLimits,
    pub lease_ttl: Duration,
    pub is_quitting: bool,
}

impl State {
    #[must_use]
    pub fn new(me: identity::Peer) -> Self {
        Self {
            me,
            lobby: discovery::Lobby(BTreeMap::new()),
            room: None,
            connected: BTreeSet::new(),
            candidates: BTreeMap::new(),
            last_dialed_at: BTreeMap::new(),
            last_handshake_at: None,
            joining_since: None,
            freenet_peers: Vec::new(),
            is_lobby_attached: false,
            display_name: identity::Username(String::new()),
            game: discovery::Config::default().game,
            held_rooms: BTreeMap::new(),
            evicted_rooms: BTreeMap::new(),
            history_size_limits: discovery::HistorySizeLimits::default(),
            lease_ttl: discovery::LeaseTimings::default().ttl,
            is_quitting: false,
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
