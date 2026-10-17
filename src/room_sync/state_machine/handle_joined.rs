use crate::identity;
use crate::room_sync;

pub fn handle_joined<Op>(
    state: &mut room_sync::state_machine::State<Op>,
    room: identity::RoomName,
    members: Vec<identity::PeerId>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    state.room = Some(room);
    state.phase = room_sync::state_machine::Phase::Fetching;
    state.buffered.clear();
    state.total = 0;
    state.parts.clear();
    state.asked.clear();
    state.source = None;
    let mut members = members;
    members.sort();
    state.members = members;
    room_sync::state_machine::fetch_next(state)
}

// no test_usage necessary
