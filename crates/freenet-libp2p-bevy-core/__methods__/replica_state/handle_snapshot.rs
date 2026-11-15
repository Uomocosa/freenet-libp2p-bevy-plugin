use crate::identity;
use crate::replica_sync;

pub fn handle_snapshot<Op>(
    state: &mut replica_sync::state_machine::ReplicaState<Op>,
    zone: identity::ZoneName,
    part: u32,
    total: u32,
    data: Vec<u8>,
) -> Vec<replica_sync::state_machine::Output<Op>> {
    tracing::debug!(target: "replica_sync", "snapshot part received part={} total={} phase={:?}", part, total, state.phase);
    if state.phase != replica_sync::state_machine::Phase::FetchingSnapshotFromZoneMember
        || state.zone.as_ref() != Some(&zone)
        || total == 0
        || part >= total
    {
        return Vec::new();
    }
    if state.member_snapshot.total_parts != total {
        state.member_snapshot.total_parts = total;
        state.member_snapshot.received_parts.clear();
    }
    state.member_snapshot.part_requested_at = None;
    state.member_snapshot.received_parts.insert(part, data);
    if let Some(next_part) =
        (0..total).find(|index| !state.member_snapshot.received_parts.contains_key(index))
    {
        let Some(peer) = state.member_snapshot.current_source.clone() else {
            return Vec::new();
        };
        return vec![
            replica_sync::state_machine::Output::FetchSnapshotPartFromZoneMember {
                peer,
                zone,
                part: next_part,
            },
        ];
    }
    let bytes = state.member_snapshot.join_received_parts();
    state.phase = replica_sync::state_machine::Phase::Ready;
    let mut outputs = vec![replica_sync::state_machine::Output::Merge { bytes }];
    for (meta, op) in state.buffered_ops.drain(..) {
        outputs.push(replica_sync::state_machine::Output::Apply { meta, op });
    }
    outputs
}

#[cfg(test)]
mod tests {
    use super::handle_snapshot;
    use crate::identity;
    use crate::replica_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let outputs = handle_snapshot(
            &mut state,
            identity::ZoneName::from("zone"),
            0,
            1,
            vec![1, 2, 3],
        );
        assert_eq!(outputs, Vec::new());
    }
}
