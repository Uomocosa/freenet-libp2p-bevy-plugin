use crate::discovery;
use discovery::state_machine::{DiscoveryState, Output};

pub fn leave(state: &mut DiscoveryState, now: discovery::UnixTime) -> Vec<Output> {
    let Some(room) = state.room.take() else {
        return Vec::new();
    };
    state.held_rooms.insert(
        room.name.clone(),
        discovery::state_machine::HeldRoomHistory {
            created_at: room.created_at,
            left_at: now,
        },
    );
    state.candidates.clear();
    state.last_dialed_at.clear();
    let mut outputs = state.broadcast_handshake();
    outputs.push(Output::Notify(discovery::Event::Left(room.name)));
    outputs
}

#[cfg(test)]
mod tests {
    use super::leave;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::DiscoveryState;

    #[test]
    fn test_usage() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        assert_eq!(leave(&mut state, discovery::UnixTime::from(1)), Vec::new());
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("r"),
            created_at: discovery::UnixTime::from(0),
            phase: discovery::JoinPhase::Ready,
            members: discovery::Members::new(),
            entry: discovery::RoomEntry::Created,
        });
        let outputs = leave(&mut state, discovery::UnixTime::from(2));
        assert!(state.room.is_none());
        assert_eq!(outputs.len(), 1);
        assert!(matches!(
            outputs.as_slice(),
            [discovery::state_machine::Output::Notify(
                discovery::Event::Left(_)
            )]
        ));
    }
}
