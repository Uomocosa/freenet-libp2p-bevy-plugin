use crate::identity;
use crate::room_sync;

pub fn handle_remote_slice_content_hashes<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
    from: &identity::PeerId,
    local: &room_sync::ReplicaSliceContentHashes,
    remote: &room_sync::ReplicaSliceContentHashes,
) -> Vec<room_sync::state_machine::Output<Op>> {
    if state.phase != room_sync::state_machine::Phase::Ready {
        tracing::debug!(target: "room_sync", "anti-entropy ignored phase={:?} reason=remote-hashes-not-ready", state.phase);
        return Vec::new();
    }
    let Some(room) = state.room.clone() else {
        tracing::debug!(target: "room_sync", "anti-entropy ignored phase={:?} reason=remote-hashes-no-room", state.phase);
        return Vec::new();
    };
    let max_fetches = state.slice_anti_entropy.max_fetches_per_peer;
    let awaiting = state
        .slice_anti_entropy
        .fetches_awaiting_reply
        .entry(from.clone())
        .or_default();
    let mut outputs = Vec::new();
    for index in room_sync::ReplicaSliceIndex::from_hash_mismatches(local, remote) {
        if awaiting.len() >= max_fetches {
            break;
        }
        if awaiting.insert(index) {
            outputs.push(room_sync::state_machine::Output::FetchReplicaSlice {
                peer: from.clone(),
                room: room.clone(),
                index,
            });
        }
    }
    tracing::debug!(target: "room_sync", "anti-entropy compare fetches={}", outputs.len());
    outputs
}

#[cfg(test)]
mod tests {
    use super::handle_remote_slice_content_hashes;
    use crate::identity;
    use crate::room_sync;
    use crate::room_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let local = room_sync::ReplicaSliceContentHashes::default();
        let outputs = handle_remote_slice_content_hashes(
            &mut state,
            &identity::PeerId::from("peer"),
            &local,
            &local,
        );
        assert_eq!(outputs, Vec::new());
    }
}
