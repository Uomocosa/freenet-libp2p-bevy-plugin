use crate::identity;
use crate::room_sync;

pub fn handle_remote<Op>(
    state: &mut room_sync::state_machine::State<Op>,
    room: &identity::RoomName,
    meta: room_sync::OpMeta,
    op: Op,
) -> Vec<room_sync::state_machine::Output<Op>> {
    if state.room.as_ref() != Some(room) {
        return Vec::new();
    }
    if state.phase == room_sync::state_machine::Phase::Ready {
        vec![room_sync::state_machine::Output::Apply { meta, op }]
    } else {
        state.buffered_ops.push((meta, op));
        Vec::new()
    }
}

// no test_usage necessary
