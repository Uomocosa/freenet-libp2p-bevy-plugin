use crate::identity;
use crate::p2p;
use crate::room_sync;

#[must_use]
pub fn handle_replica_slice_requested<Op>(
    state: &room_sync::state_machine::ReplicaState<Op>,
    id: p2p::ReplyId,
    room: identity::RoomName,
    index: room_sync::ReplicaSliceIndex,
) -> Vec<room_sync::state_machine::Output<Op>> {
    if state.room.as_ref() != Some(&room) {
        return Vec::new();
    }
    vec![room_sync::state_machine::Output::SendReplicaSlice { id, room, index }]
}

#[cfg(test)]
mod tests {
    use super::handle_replica_slice_requested;
    use crate::identity;
    use crate::p2p;
    use crate::room_sync;
    use crate::room_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let state = ReplicaState::<u8>::default();
        let outputs = handle_replica_slice_requested(
            &state,
            p2p::ReplyId(1),
            identity::RoomName::from("room"),
            room_sync::ReplicaSliceIndex(0),
        );
        assert_eq!(outputs, Vec::new());
    }
}
