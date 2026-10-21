use crate::identity;
use crate::room_sync;

pub fn handle_remote_slice_content_hashes<Op>(
    state: &mut room_sync::state_machine::State<Op>,
    from: &identity::PeerId,
    local: &room_sync::ReplicaSliceContentHashes,
    remote: &room_sync::ReplicaSliceContentHashes,
) -> Vec<room_sync::state_machine::Output<Op>> {
    if state.phase != room_sync::state_machine::Phase::Ready {
        return Vec::new();
    }
    let Some(room) = state.room.clone() else {
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
    outputs
}

// no test_usage necessary
