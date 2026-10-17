use crate::identity;
use crate::p2p;
use crate::room_sync;

#[must_use]
pub fn handle_fetch<O>(
    state: &room_sync::state_machine::State<O>,
    id: p2p::ReplyId,
    room: identity::RoomName,
    part: u32,
) -> Vec<room_sync::state_machine::Output<O>> {
    if state.room.is_none() {
        return Vec::new();
    }
    vec![room_sync::state_machine::Output::Snapshot { id, room, part }]
}

// no test_usage necessary
