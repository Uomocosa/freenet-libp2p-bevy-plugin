use crate::discovery;
use discovery::state_machine::{Output, State};

#[must_use]
pub fn tick(state: &mut State, now: discovery::UnixTime) -> Vec<Output> {
    let mut outputs = discovery::state_machine::dial_candidates(state, now);
    let handshake_due = state
        .last_handshake
        .is_none_or(|last| now.since(last) >= discovery::state_machine::HANDSHAKE_EVERY);
    if handshake_due && state.room.is_some() {
        outputs.extend(discovery::state_machine::broadcast_handshake(state));
        state.last_handshake = Some(now);
    }
    outputs.extend(discovery::state_machine::prune_members(state, now));
    outputs.extend(discovery::state_machine::wait_for_contract_self_heal(
        state, now,
    ));
    outputs
}

#[cfg(test)]
mod tests {
    use super::tick;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.connected.insert(identity::PeerId::from("a"));
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("r"),
            created_at: discovery::UnixTime::from_secs(0),
            phase: discovery::RoomPhase::Ready,
            members: discovery::Members::new(),
        });
        let mut outputs = tick(&mut state, discovery::UnixTime::from_secs(100));
        outputs.extend(tick(&mut state, discovery::UnixTime::from_secs(101)));
        assert_eq!(
            state.last_handshake,
            Some(discovery::UnixTime::from_secs(100))
        );
        assert_eq!(outputs.len(), 1);
    }
}
