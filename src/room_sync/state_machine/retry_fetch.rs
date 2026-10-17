use crate::room_sync;

// needed helper: no member left to answer; accept what we already have
fn give_up<Op>(
    state: &mut room_sync::state_machine::State<Op>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    state.phase = room_sync::state_machine::Phase::Ready;
    let mut outputs = Vec::new();
    for (meta, op) in state.buffered.drain(..) {
        outputs.push(room_sync::state_machine::Output::Apply { meta, op });
    }
    outputs
}

pub fn retry_fetch<Op>(
    state: &mut room_sync::state_machine::State<Op>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    if state.phase != room_sync::state_machine::Phase::Fetching {
        return Vec::new();
    }
    let next = room_sync::state_machine::fetch_next(state);
    if !next.is_empty() {
        return next;
    }
    let (Some(peer), Some(room)) = (state.source.clone(), state.room.clone()) else {
        return give_up(state);
    };
    let hole = state.parts.iter().position(Option::is_none);
    let part = u32::try_from(hole.unwrap_or(state.parts.len())).unwrap_or(u32::MAX);
    vec![room_sync::state_machine::Output::Fetch { peer, room, part }]
}

// no test_usage necessary
