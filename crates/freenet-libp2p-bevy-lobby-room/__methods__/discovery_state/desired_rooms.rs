use std::collections::BTreeSet;

use crate::discovery;
use crate::identity;
use discovery::state_machine::DiscoveryState;

#[must_use]
pub fn desired_rooms(state: &DiscoveryState) -> BTreeSet<identity::RoomName> {
    let mut rooms: BTreeSet<identity::RoomName> = state.lobby.keys().cloned().collect();
    if let Some(joined) = state.room.as_ref() {
        rooms.insert(joined.name.clone());
    }
    rooms
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::desired_rooms;
    use crate::discovery;
    use crate::identity;
    use crate::time;
    use discovery::state_machine::DiscoveryState;

    #[test]
    fn test_usage() {
        let room = identity::RoomName::from("room-a");
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            room.clone(),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::from_secs(10),
                members: BTreeMap::new(),
            },
        );
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.lobby = lobby;
        assert_eq!(desired_rooms(&state), BTreeSet::from([room]));
        let empty = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        assert!(desired_rooms(&empty).is_empty());
    }
}
