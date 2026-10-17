use crate::room_sync;

pub fn tick<O>(
    state: &mut room_sync::state_machine::State<O>,
) -> Vec<room_sync::state_machine::Output<O>> {
    if state.phase != room_sync::state_machine::Phase::Fetching {
        return Vec::new();
    }
    if state.parts.iter().any(Option::is_none) {
        return Vec::new();
    }
    let retry = room_sync::state_machine::fetch_next(state);
    if !retry.is_empty() {
        return retry;
    }
    state.phase = room_sync::state_machine::Phase::Ready;
    let mut outputs = Vec::new();
    for (meta, op) in state.buffered.drain(..) {
        outputs.push(room_sync::state_machine::Output::Apply { meta, op });
    }
    outputs
}

// no test_usage necessary
