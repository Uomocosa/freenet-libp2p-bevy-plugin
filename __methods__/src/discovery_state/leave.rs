use crate::discovery;
use crate::time;
use discovery::state_machine::{DiscoveryState, Output};

pub fn leave(state: &mut DiscoveryState, now: time::UnixTime) -> Vec<Output> {
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
    use crate::time;
    use discovery::state_machine::DiscoveryState;

    #[test]
    fn test_usage() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        assert_eq!(leave(&mut state, time::UnixTime::from_secs(1)), Vec::new());
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("r"),
            created_at: time::UnixTime::EPOCH,
            phase: discovery::JoinPhase::Ready,
            members: discovery::Members::new(),
            how_entered: discovery::HowRoomWasEntered::Created,
        });
        let outputs = leave(&mut state, time::UnixTime::from_secs(2));
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
