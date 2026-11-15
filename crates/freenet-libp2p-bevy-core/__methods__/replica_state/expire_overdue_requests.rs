use crate::replica_sync;

pub fn expire_overdue_requests<Op>(
    state: &mut replica_sync::state_machine::ReplicaState<Op>,
    now: replica_sync::ElapsedSinceAppStart,
) -> Vec<replica_sync::state_machine::Output<Op>> {
    let mut outputs = state.expire_overdue_snapshot_part_request(now);
    outputs.extend(state.expire_overdue_replica_slice_requests(now));
    outputs.extend(state.expire_overdue_history_record_requests(now));
    outputs
}

#[cfg(test)]
mod tests {
    use super::expire_overdue_requests;
    use crate::replica_sync;
    use crate::replica_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let now = replica_sync::ElapsedSinceAppStart::default();
        assert_eq!(expire_overdue_requests(&mut state, now), Vec::new());
    }
}
