use crate::spectator;
use crate::time;

pub fn expire_slice_requests<Op, Snap>(
    state: &mut spectator::state_machine::SpectatorState<Op, Snap>,
    now: time::UnixTime,
) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    let before = state.pending_slice_requests.len();
    state.pending_slice_requests.retain(|_, requested| {
        now.elapsed_since(*requested) < spectator::state_machine::SLICE_REPAIR_TIMEOUT
    });
    if state.pending_slice_requests.len() == before || !state.pending_slice_requests.is_empty() {
        return Vec::new();
    }
    if state.phase != spectator::state_machine::SpectatePhase::Live {
        return Vec::new();
    }
    vec![spectator::state_machine::SpectateOutput::RepairFinished]
}

#[cfg(test)]
mod tests {
    use super::expire_slice_requests;
    use crate::replica_sync;
    use crate::spectator;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut state =
            testing::spectator_state(None, spectator::state_machine::SpectatePhase::Live);
        state
            .pending_slice_requests
            .insert(replica_sync::ReplicaSliceIndex(1), time::UnixTime::EPOCH);
        assert_eq!(
            expire_slice_requests(&mut state, time::UnixTime::from_secs(9)),
            Vec::new()
        );
        assert_eq!(state.pending_slice_requests.len(), 1);
        assert_eq!(
            expire_slice_requests(&mut state, time::UnixTime::from_secs(10)),
            vec![spectator::state_machine::SpectateOutput::RepairFinished]
        );
        assert!(state.pending_slice_requests.is_empty());
    }

    #[test]
    fn a_remaining_pending_request_keeps_the_repair_open() {
        let mut state =
            testing::spectator_state(None, spectator::state_machine::SpectatePhase::Live);
        state
            .pending_slice_requests
            .insert(replica_sync::ReplicaSliceIndex(1), time::UnixTime::EPOCH);
        state.pending_slice_requests.insert(
            replica_sync::ReplicaSliceIndex(2),
            time::UnixTime::from_secs(9),
        );
        assert_eq!(
            expire_slice_requests(&mut state, time::UnixTime::from_secs(10)),
            Vec::new()
        );
        assert_eq!(state.pending_slice_requests.len(), 1);
    }

    #[test]
    fn no_pending_request_reports_nothing() {
        let mut state =
            testing::spectator_state(None, spectator::state_machine::SpectatePhase::Live);
        assert_eq!(
            expire_slice_requests(&mut state, time::UnixTime::from_secs(10)),
            Vec::new()
        );
    }
}
