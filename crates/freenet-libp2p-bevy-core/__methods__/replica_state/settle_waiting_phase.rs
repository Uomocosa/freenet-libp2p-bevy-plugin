use crate::replica_sync;

pub fn settle_waiting_phase<Op>(
    state: &mut replica_sync::state_machine::ReplicaState<Op>,
    outputs_so_far: &[replica_sync::state_machine::Output<Op>],
) -> Vec<replica_sync::state_machine::Output<Op>> {
    match state.phase {
        replica_sync::state_machine::Phase::FetchingSnapshotFromZoneMember => {
            if state.zone.is_none()
                || state.member_snapshot.part_requested_at.is_some()
                || outputs_so_far.iter().any(|output| {
                    matches!(
                        output,
                        replica_sync::state_machine::Output::FetchSnapshotPartFromZoneMember { .. }
                    )
                })
            {
                return Vec::new();
            }
            let next = state.fetch_next();
            if !next.is_empty() {
                return next;
            }
            state.finish_member_fetch()
        }
        replica_sync::state_machine::Phase::FetchingHistoryFromKademlia => {
            if state.kademlia_history.pending_fetch.is_some()
                || outputs_so_far.iter().any(|output| {
                    matches!(
                        output,
                        replica_sync::state_machine::Output::GetHistoryRecord { .. }
                    )
                })
            {
                return Vec::new();
            }
            state.phase = replica_sync::state_machine::Phase::Ready;
            state
                .buffered_ops
                .drain(..)
                .map(|(meta, op)| replica_sync::state_machine::Output::Apply { meta, op })
                .collect()
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::settle_waiting_phase;
    use crate::replica_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        assert_eq!(settle_waiting_phase(&mut state, &[]), Vec::new());
    }
}
