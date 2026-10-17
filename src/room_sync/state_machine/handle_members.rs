use crate::identity;
use crate::room_sync;

pub fn handle_members<O>(
    state: &mut room_sync::state_machine::State<O>,
    members: Vec<identity::PeerId>,
) -> Vec<room_sync::state_machine::Output<O>> {
    let mut members = members;
    members.sort();
    state.members = members;
    if state.phase == room_sync::state_machine::Phase::Fetching {
        room_sync::state_machine::fetch_next(state)
    } else {
        Vec::new()
    }
}

// no test_usage necessary
