use crate::room_sync;

pub fn fetch_next<O>(
    state: &mut room_sync::state_machine::State<O>,
) -> Vec<room_sync::state_machine::Output<O>> {
    let Some(room) = state.room.clone() else {
        return Vec::new();
    };
    let peer = state
        .members
        .iter()
        .find(|member| !state.asked.contains(member))
        .cloned();
    let Some(peer) = peer else {
        return Vec::new();
    };
    state.asked.push(peer.clone());
    state.source = Some(peer.clone());
    vec![room_sync::state_machine::Output::Fetch {
        peer,
        room,
        part: 0,
    }]
}

// no test_usage necessary
