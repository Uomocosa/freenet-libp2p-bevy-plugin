use crate::identity;
use crate::p2p;
use crate::replica_sync;

#[must_use]
pub fn handle_replica_slice_requested<Op>(
    state: &replica_sync::state_machine::ReplicaState<Op>,
    id: p2p::ReplyId,
    zone: identity::ZoneName,
    index: replica_sync::ReplicaSliceIndex,
) -> Vec<replica_sync::state_machine::Output<Op>> {
    if state.zone.as_ref() != Some(&zone) {
        return Vec::new();
    }
    vec![replica_sync::state_machine::Output::SendReplicaSlice { id, zone, index }]
}

#[cfg(test)]
mod tests {
    use super::handle_replica_slice_requested;
    use crate::identity;
    use crate::p2p;
    use crate::replica_sync;
    use crate::replica_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let state = ReplicaState::<u8>::default();
        let outputs = handle_replica_slice_requested(
            &state,
            p2p::ReplyId(1),
            identity::ZoneName::from("zone"),
            replica_sync::ReplicaSliceIndex(0),
        );
        assert_eq!(outputs, Vec::new());
    }
}
