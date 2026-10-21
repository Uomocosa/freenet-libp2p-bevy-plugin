use crate::room_sync;

pub fn tick<Op>(
    state: &mut room_sync::state_machine::State<Op>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    if state.phase != room_sync::state_machine::Phase::FetchingSnapshotFromRoomMember {
        return Vec::new();
    }
    let is_missing_parts = state.member_snapshot.total_parts > 0
        && (0..state.member_snapshot.total_parts)
            .any(|index| !state.member_snapshot.received_parts.contains_key(&index));
    if is_missing_parts {
        return Vec::new();
    }
    let retry = room_sync::state_machine::fetch_next(state);
    if !retry.is_empty() {
        return retry;
    }
    if state.kademlia_history.is_enabled {
        return room_sync::state_machine::start_kademlia_history_fetch(state);
    }
    state.phase = room_sync::state_machine::Phase::Ready;
    let mut outputs = Vec::new();
    for (meta, op) in state.buffered_ops.drain(..) {
        outputs.push(room_sync::state_machine::Output::Apply { meta, op });
    }
    outputs
}

// no test_usage necessary
