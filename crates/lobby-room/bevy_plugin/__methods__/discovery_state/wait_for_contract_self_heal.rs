use crate::discovery;
use crate::time;
use discovery::state_machine::{DiscoveryState, Output};

pub fn wait_for_contract_self_heal(state: &mut DiscoveryState, now: time::UnixTime) -> Vec<Output> {
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
    use super::wait_for_contract_self_heal;
    use crate::discovery;
    use crate::testing;
    use crate::time;
    use discovery::state_machine::Output;

    #[test]
    fn test_usage() {
        let mut waiting = testing::joining_state("me", "r", 100);
        assert_eq!(
            wait_for_contract_self_heal(&mut waiting, time::UnixTime::from_secs(1)),
            Vec::new()
        );
        assert!(matches!(
            testing::join_phase(&waiting),
            Some(discovery::JoinPhase::Joining { .. })
        ));

        let mut healed = testing::joining_state("me", "r", 100);
        healed.lobby = testing::lobby_with(vec![(
            "r",
            testing::room(discovery::RoomSize::DEFAULT, 100, Vec::new()),
        )]);
        let outputs = wait_for_contract_self_heal(&mut healed, time::UnixTime::from_secs(1));
        assert_eq!(
            outputs,
            vec![Output::Notify(discovery::Event::RoomReady(
                discovery::RoomName::from("r")
            ))]
        );
        assert_eq!(
            testing::join_phase(&healed),
            Some(discovery::JoinPhase::Ready)
        );
    }

    #[test]
    fn times_out_and_proceeds() {
        let mut state = testing::joining_state("me", "r", 100);
        let outputs = wait_for_contract_self_heal(
            &mut state,
            time::UnixTime::from_secs(discovery::state_machine::JOIN_HEAL_TIMEOUT.as_secs()),
        );
        assert_eq!(
            outputs,
            vec![Output::Notify(discovery::Event::RoomReady(
                discovery::RoomName::from("r")
            ))]
        );
        assert_eq!(
            testing::join_phase(&state),
            Some(discovery::JoinPhase::Ready)
        );
    }

    #[test]
    fn does_nothing_without_a_pending_join() {
        let mut state = testing::joining_state("me", "r", 100);
        state.lobby = testing::lobby_with(vec![(
            "r",
            testing::room(discovery::RoomSize::DEFAULT, 100, Vec::new()),
        )]);
        if let Some(room) = state.room.as_mut() {
            room.phase = discovery::JoinPhase::Ready;
        }
        assert_eq!(
            wait_for_contract_self_heal(&mut state, time::UnixTime::from_secs(1)),
            Vec::new()
        );
        assert_eq!(
            testing::join_phase(&state),
            Some(discovery::JoinPhase::Ready)
        );
    }
}
