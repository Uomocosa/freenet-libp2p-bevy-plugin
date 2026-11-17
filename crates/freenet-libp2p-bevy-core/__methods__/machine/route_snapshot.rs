use crate::p2p;
use crate::spectator;
use crate::time;

pub fn route_snapshot<Op: Clone>(
    machine: &mut spectator::Machine<Op>,
    history_prefix: &p2p::HistoryKeyPrefix,
    snapshot: Vec<u8>,
    now: time::UnixTime,
) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    let slot = machine
        .sessions
        .iter()
        .find(|(_, session)| session.history_prefix.as_ref() == Some(history_prefix))
        .map(|(slot, _)| *slot);
    let Some(slot) = slot else {
        return Vec::new();
    };
    machine.update_session(
        slot,
        spectator::state_machine::SpectateInput::Snapshot {
            history_prefix: history_prefix.clone(),
            snapshot,
        },
        now,
    )
}

#[cfg(test)]
mod tests {
    use super::route_snapshot;
    use crate::identity;
    use crate::p2p;
    use crate::spectator;
    use crate::time;

    fn machine() -> spectator::Machine<u8> {
        spectator::Machine::new(identity::GameId::from("game"))
    }

    #[test]
    fn test_usage() {
        let mut machine = machine();
        let prefix = p2p::HistoryKeyPrefix::from("prefix");
        assert_eq!(
            route_snapshot(&mut machine, &prefix, vec![1], time::UnixTime::EPOCH),
            Vec::new()
        );
    }

    #[test]
    fn a_snapshot_reaches_the_session_that_fetched_its_history() {
        let mut machine = machine();
        let prefix = p2p::HistoryKeyPrefix::from("prefix");
        let mut session = spectator::state_machine::SpectatorState::<u8, Vec<u8>>::new(
            identity::GameId::from("game"),
        );
        session.start_spectating(
            &identity::ZoneName::from("zone"),
            prefix.clone(),
            time::UnixTime::EPOCH,
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        machine
            .sessions
            .insert(spectator::SpectateSlot::Stream, session);
        let outputs = route_snapshot(&mut machine, &prefix, vec![7], time::UnixTime::EPOCH);
        assert!(matches!(
            outputs.as_slice(),
            [spectator::MachineOutput::Session {
                slot: spectator::SpectateSlot::Stream,
                output: spectator::state_machine::SpectateOutput::Merge(_),
            }]
        ));
    }
}
