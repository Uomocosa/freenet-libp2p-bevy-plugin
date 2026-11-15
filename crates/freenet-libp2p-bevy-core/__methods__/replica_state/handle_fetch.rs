use crate::identity;
use crate::p2p;
use crate::replica_sync;

#[must_use]
pub fn handle_fetch<Op>(
    state: &replica_sync::state_machine::ReplicaState<Op>,
    id: p2p::ReplyId,
    zone: identity::ZoneName,
    part: u32,
) -> Vec<replica_sync::state_machine::Output<Op>> {
    tracing::debug!(target: "replica_sync", "snapshot fetch requested zone={:?} part={}", zone, part);
    if state.zone.is_none() {
        return Vec::new();
    }
    vec![replica_sync::state_machine::Output::SendSnapshotPartToZoneMember { id, zone, part }]
}

#[cfg(test)]
mod tests {
    use super::handle_fetch;
    use crate::identity;
    use crate::p2p;
    use crate::replica_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let state = ReplicaState::<u8>::default();
        let outputs = handle_fetch(&state, p2p::ReplyId(1), identity::ZoneName::from("zone"), 0);
        assert_eq!(outputs, Vec::new());
    }
}
