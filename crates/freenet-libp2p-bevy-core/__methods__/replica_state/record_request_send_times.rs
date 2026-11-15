use crate::replica_sync;

pub fn record_request_send_times<Op>(
    state: &mut replica_sync::state_machine::ReplicaState<Op>,
    outputs: &[replica_sync::state_machine::Output<Op>],
    now: replica_sync::ElapsedSinceAppStart,
) {
    for output in outputs {
        match output {
            replica_sync::state_machine::Output::FetchSnapshotPartFromZoneMember { .. } => {
                state.member_snapshot.part_requested_at = Some(now);
            }
            replica_sync::state_machine::Output::GetHistoryRecord { .. } => {
                if let Some(pending) = state.kademlia_history.pending_fetch.as_mut() {
                    pending.records_requested_at = Some(now);
                }
            }
            replica_sync::state_machine::Output::FetchReplicaSlice { peer, index, .. } => {
                state
                    .slice_anti_entropy
                    .fetches_awaiting_reply
                    .entry(peer.clone())
                    .or_default()
                    .insert(*index, now);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::record_request_send_times;
    use crate::replica_sync;
    use crate::replica_sync::state_machine::{Phase, ReplicaState};

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        record_request_send_times(
            &mut state,
            &[],
            replica_sync::ElapsedSinceAppStart::default(),
        );
        assert_eq!(state.phase, Phase::Idle);
    }
}
