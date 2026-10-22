use crate::room_sync;

#[must_use]
pub fn publish_slice_content_hashes<Op>(
    state: &room_sync::state_machine::ReplicaState<Op>,
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

#[cfg(test)]
mod tests {
    use super::publish_slice_content_hashes;
    use crate::room_sync;
    use crate::room_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let state = ReplicaState::<u8>::default();
        let outputs =
            publish_slice_content_hashes(&state, room_sync::ReplicaSliceContentHashes::default());
        assert_eq!(outputs, Vec::new());
    }
}
