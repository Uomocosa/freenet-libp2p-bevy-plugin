use crate::identity;
use crate::p2p;
use crate::room_sync;

#[must_use]
pub fn handle_replica_slice_requested<Op>(
    state: &room_sync::state_machine::State<Op>,
    id: p2p::ReplyId,
    room: identity::RoomName,
    index: room_sync::ReplicaSliceIndex,
) -> Vec<room_sync::state_machine::Output<Op>> {
    if state.room.as_ref() != Some(&room) {
        return Vec::new();
    }
    vec![room_sync::state_machine::Output::SendReplicaSlice { id, room, index }]
}

// no test_usage necessary
