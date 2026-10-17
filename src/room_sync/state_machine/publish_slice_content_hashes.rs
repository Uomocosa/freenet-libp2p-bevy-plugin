use crate::room_sync;

#[must_use]
pub fn publish_slice_content_hashes<Op>(
    state: &room_sync::state_machine::State<Op>,
    local: room_sync::ReplicaSliceContentHashes,
) -> Vec<room_sync::state_machine::Output<Op>> {
    let Some(room) = state.room.clone() else {
        return Vec::new();
    };
    if state.phase != room_sync::state_machine::Phase::Ready {
        return Vec::new();
    }
    vec![
        room_sync::state_machine::Output::PublishSliceContentHashes {
            room,
            hashes: local,
        },
    ]
}

// no test_usage necessary
