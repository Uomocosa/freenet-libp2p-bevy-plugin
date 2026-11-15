use crate::identity;
use crate::replica_sync;

pub fn handle_replica_slice_received<Op>(
    state: &mut replica_sync::state_machine::ReplicaState<Op>,
    from: &identity::PeerId,
    zone: &identity::ZoneName,
    index: replica_sync::ReplicaSliceIndex,
    data: Vec<u8>,
) -> Vec<replica_sync::state_machine::Output<Op>> {
    tracing::debug!(target: "replica_sync", "anti-entropy slice from={from} index={} bytes={}", *index, data.len());
    if let Some(awaiting) = state
        .slice_anti_entropy
        .fetches_awaiting_reply
        .get_mut(from)
    {
        awaiting.remove(&index);
    }
    if state.zone.as_ref() != Some(zone) {
        return Vec::new();
    }
    tracing::debug!(target: "replica_sync", "anti-entropy slice index={} merged", *index);
    vec![replica_sync::state_machine::Output::Merge { bytes: data }]
}

#[cfg(test)]
mod tests {
    use super::handle_replica_slice_received;
    use crate::identity;
    use crate::replica_sync;
    use crate::replica_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let outputs = handle_replica_slice_received(
            &mut state,
            &identity::PeerId::from("peer"),
            &identity::ZoneName::from("zone"),
            replica_sync::ReplicaSliceIndex(0),
            Vec::new(),
        );
        assert_eq!(outputs, Vec::new());
    }
}
