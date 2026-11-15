use crate::replica_sync;

#[must_use]
pub fn publish_slice_content_hashes<Op>(
    state: &replica_sync::state_machine::ReplicaState<Op>,
    local: replica_sync::ReplicaSliceContentHashes,
) -> Vec<replica_sync::state_machine::Output<Op>> {
    let Some(room) = state.room.clone() else {
        tracing::debug!(target: "replica_sync", "anti-entropy ignored phase={:?} reason=publish-no-room", state.phase);
        return Vec::new();
    };
    if state.phase != replica_sync::state_machine::Phase::Ready {
        tracing::debug!(target: "replica_sync", "anti-entropy ignored phase={:?} reason=publish-not-ready", state.phase);
        return Vec::new();
    }
    vec![
        replica_sync::state_machine::Output::PublishSliceContentHashes {
            room,
            hashes: local,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::publish_slice_content_hashes;
    use crate::replica_sync;
    use crate::replica_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let state = ReplicaState::<u8>::default();
        let outputs = publish_slice_content_hashes(
            &state,
            replica_sync::ReplicaSliceContentHashes::default(),
        );
        assert_eq!(outputs, Vec::new());
    }
}
