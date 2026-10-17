use crate::identity;
use crate::room_sync;

pub fn handle_members<Op>(
    state: &mut room_sync::state_machine::State<Op>,
    members: Vec<identity::PeerId>,
) -> Vec<room_sync::state_machine::Output<Op>> {
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
