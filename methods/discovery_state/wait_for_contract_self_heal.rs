use crate::discovery;
use discovery::state_machine::{DiscoveryState, Output};

pub fn wait_for_contract_self_heal(
    state: &mut DiscoveryState,
    now: discovery::UnixTime,
) -> Vec<Output> {
    let Some(room) = state.room.as_ref() else {
        return Vec::new();
    };
    let discovery::JoinPhase::Joining { since } = room.phase else {
        return Vec::new();
    };
    let name = room.name.clone();
    let created_at = room.created_at;
    let healed = state
        .lobby
        .get(&name)
        .is_some_and(|record| record.created_at == created_at);
    let timed_out = now.elapsed_since(since) >= discovery::state_machine::JOIN_HEAL_TIMEOUT;
    if !healed && !timed_out {
        return Vec::new();
    }
    if !healed {
        tracing::warn!(target: "discovery", room = %name.as_str(), "join self-heal timed out; proceeding");
    }
    if let Some(room) = state.room.as_mut() {
        room.phase = discovery::JoinPhase::Ready;
    }
    vec![Output::Notify(discovery::Event::RoomReady(name))]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::wait_for_contract_self_heal;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::{DiscoveryState, Output};

    fn state_in(created_at: u64) -> DiscoveryState {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("r"),
            created_at: discovery::UnixTime::from(created_at),
            phase: discovery::JoinPhase::Joining {
                since: discovery::UnixTime::from(0),
            },
            members: discovery::Members::new(),
            entry: discovery::RoomEntry::Created,
        });
        state
    }

    fn with_record(state: &mut DiscoveryState, created_at: u64) {
        state.lobby.insert(
            identity::RoomName::from("r"),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from(created_at),
                members: BTreeMap::new(),
            },
        );
    }

    fn phase(state: &DiscoveryState) -> Option<discovery::JoinPhase> {
        state.room.as_ref().map(|room| room.phase)
    }

    #[test]
    fn test_usage() {
        let mut waiting = state_in(100);
        assert_eq!(
            wait_for_contract_self_heal(&mut waiting, discovery::UnixTime::from(1)),
            Vec::new()
        );
        assert!(matches!(
            phase(&waiting),
            Some(discovery::JoinPhase::Joining { .. })
        ));

        let mut healed = state_in(100);
        with_record(&mut healed, 100);
        let outputs = wait_for_contract_self_heal(&mut healed, discovery::UnixTime::from(1));
        assert_eq!(
            outputs,
            vec![Output::Notify(discovery::Event::RoomReady(
                identity::RoomName::from("r")
            ))]
        );
        assert_eq!(phase(&healed), Some(discovery::JoinPhase::Ready));
    }

    #[test]
    fn times_out_and_proceeds() {
        let mut state = state_in(100);
        let outputs = wait_for_contract_self_heal(
            &mut state,
            discovery::UnixTime::from(discovery::state_machine::JOIN_HEAL_TIMEOUT.as_secs()),
        );
        assert_eq!(
            outputs,
            vec![Output::Notify(discovery::Event::RoomReady(
                identity::RoomName::from("r")
            ))]
        );
        assert_eq!(phase(&state), Some(discovery::JoinPhase::Ready));
    }

    #[test]
    fn does_nothing_without_a_pending_join() {
        let mut state = state_in(100);
        with_record(&mut state, 100);
        if let Some(room) = state.room.as_mut() {
            room.phase = discovery::JoinPhase::Ready;
        }
        assert_eq!(
            wait_for_contract_self_heal(&mut state, discovery::UnixTime::from(1)),
            Vec::new()
        );
        assert_eq!(phase(&state), Some(discovery::JoinPhase::Ready));
    }
}
