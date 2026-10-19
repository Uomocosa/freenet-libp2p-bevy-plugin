use crate::discovery;
use discovery::state_machine::State;

pub fn align_room_created_at(state: &mut State) {
    let Some(room) = state.room.as_ref() else {
        return;
    };
    let Some(record) = state.lobby.get(&room.name) else {
        return;
    };
    let lobby_created_at = record.created_at;
    if let Some(room) = state.room.as_mut() {
        room.created_at = lobby_created_at;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::align_room_created_at;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let room = identity::RoomName::from("r");
        state.room = Some(discovery::JoinedRoom {
            name: room.clone(),
            created_at: discovery::UnixTime::from_secs(500),
            phase: discovery::RoomPhase::Joining,
            members: discovery::Members::new(),
        });
        state.lobby.insert(
            room,
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from_secs(100),
                members: BTreeMap::new(),
            },
        );
        align_room_created_at(&mut state);
        assert_eq!(
            state.room.as_ref().map(|room| room.created_at),
            Some(discovery::UnixTime::from_secs(100))
        );
    }

    #[test]
    fn leaves_room_untouched_without_a_record() {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("r"),
            created_at: discovery::UnixTime::from_secs(500),
            phase: discovery::RoomPhase::Joining,
            members: discovery::Members::new(),
        });
        align_room_created_at(&mut state);
        assert_eq!(
            state.room.as_ref().map(|room| room.created_at),
            Some(discovery::UnixTime::from_secs(500))
        );
    }
}
