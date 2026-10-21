use crate::identity;
use crate::room_sync;

pub fn handle_replica_slice_received<Op>(
    state: &mut room_sync::state_machine::State<Op>,
    from: &identity::PeerId,
    room: &identity::RoomName,
    index: room_sync::ReplicaSliceIndex,
    data: Vec<u8>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    if let Some(awaiting) = state
        .slice_anti_entropy
        .fetches_awaiting_reply
        .get_mut(from)
    {
        awaiting.remove(&index);
    }
    if state.room.as_ref() != Some(room) {
        return Vec::new();
    }
    vec![room_sync::state_machine::Output::Merge { bytes: data }]
}

// no test_usage necessary
