use crate::identity;
use crate::room_sync;

pub fn handle_remote<O>(
    state: &mut room_sync::state_machine::State<O>,
    room: &identity::RoomName,
    meta: room_sync::OpMeta,
    op: O,
) -> Vec<room_sync::state_machine::Output<O>> {
    if state.room.as_ref() != Some(room) {
        return Vec::new();
    }
    if state.phase == room_sync::state_machine::Phase::Ready {
        vec![room_sync::state_machine::Output::Apply { meta, op }]
    } else {
        state.buffered.push((meta, op));
        Vec::new()
    }
}

// no test_usage necessary
