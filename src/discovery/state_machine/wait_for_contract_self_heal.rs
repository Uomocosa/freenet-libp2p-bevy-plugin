use crate::discovery;
use discovery::state_machine::{Output, State};

pub fn wait_for_contract_self_heal(state: &mut State, now: discovery::UnixTime) -> Vec<Output> {
    let Some(since) = state.joining_since else {
        return Vec::new();
    };
    let Some(room) = state.room.as_ref() else {
        return Vec::new();
    };
    let name = room.name.clone();
    let healed = state
        .lobby
        .get(&name)
        .is_some_and(|record| record.created_at == room.created_at);
    let timed_out = now.since(since) >= discovery::state_machine::JOIN_HEAL_TIMEOUT;
    if !healed && !timed_out {
        return Vec::new();
    }
    if !healed {
        tracing::warn!(target: "discovery", room = %name.as_str(), "join self-heal timed out; proceeding");
    }
    state.joining_since = None;
    if let Some(room) = state.room.as_mut() {
        room.phase = discovery::RoomPhase::Ready;
    }
    vec![Output::Notify(discovery::Event::RoomReady(name))]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::wait_for_contract_self_heal;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::{Output, State};

    fn state_in(created_at: u64) -> State {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("r"),
            created_at: discovery::UnixTime::from_secs(created_at),
            phase: discovery::RoomPhase::Joining,
            members: discovery::Members::new(),
        });
        state.joining_since = Some(discovery::UnixTime::from_secs(0));
        state
    }

    fn with_record(state: &mut State, created_at: u64) {
        state.lobby.insert(
            identity::RoomName::from("r"),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from_secs(created_at),
                members: BTreeMap::new(),
            },
        );
    }

    fn phase(state: &State) -> Option<discovery::RoomPhase> {
        state.room.as_ref().map(|room| room.phase)
    }

    #[test]
    fn test_usage() {
        let mut waiting = state_in(100);
        assert_eq!(
            wait_for_contract_self_heal(&mut waiting, discovery::UnixTime::from_secs(1)),
            Vec::new()
        );
        assert_eq!(phase(&waiting), Some(discovery::RoomPhase::Joining));

        let mut healed = state_in(100);
        with_record(&mut healed, 100);
        let outputs = wait_for_contract_self_heal(&mut healed, discovery::UnixTime::from_secs(1));
        assert_eq!(
            outputs,
            vec![Output::Notify(discovery::Event::RoomReady(
                identity::RoomName::from("r")
            ))]
        );
        assert_eq!(phase(&healed), Some(discovery::RoomPhase::Ready));
        assert!(healed.joining_since.is_none());
    }

    #[test]
    fn times_out_and_proceeds() {
        let mut state = state_in(100);
        state.joining_since = Some(discovery::UnixTime::from_secs(0));
        let outputs = wait_for_contract_self_heal(
            &mut state,
            discovery::UnixTime::from_secs(discovery::state_machine::JOIN_HEAL_TIMEOUT.as_secs()),
        );
        assert_eq!(
            outputs,
            vec![Output::Notify(discovery::Event::RoomReady(
                identity::RoomName::from("r")
            ))]
        );
        assert_eq!(phase(&state), Some(discovery::RoomPhase::Ready));
    }

    #[test]
    fn does_nothing_without_a_pending_join() {
        let mut state = state_in(100);
        with_record(&mut state, 100);
        state.joining_since = None;
        assert_eq!(
            wait_for_contract_self_heal(&mut state, discovery::UnixTime::from_secs(1)),
            Vec::new()
        );
        assert_eq!(phase(&state), Some(discovery::RoomPhase::Joining));
    }
}
