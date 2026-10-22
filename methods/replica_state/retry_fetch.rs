use crate::room_sync;

// needed helper: no member left to answer; fall back to Kademlia or accept what we already have
fn finish_member_fetch<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    if state.kademlia_history.is_enabled {
        return state.start_kademlia_history_fetch();
    }
    state.phase = room_sync::state_machine::Phase::Ready;
    let mut outputs = Vec::new();
    for (meta, op) in state.buffered_ops.drain(..) {
        outputs.push(room_sync::state_machine::Output::Apply { meta, op });
    }
    outputs
}

pub fn retry_fetch<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    if state.phase != room_sync::state_machine::Phase::FetchingSnapshotFromRoomMember {
        return Vec::new();
    }
    let next = state.fetch_next();
    if !next.is_empty() {
        return next;
    }
    let (Some(peer), Some(room)) = (
        state.member_snapshot.current_source.clone(),
        state.room.clone(),
    ) else {
        return finish_member_fetch(state);
    };
    if state.member_snapshot.total_parts == 0 {
        return finish_member_fetch(state);
    }
    let part = (0..state.member_snapshot.total_parts)
        .find(|index| !state.member_snapshot.received_parts.contains_key(index))
        .unwrap_or(state.member_snapshot.total_parts);
    vec![room_sync::state_machine::Output::FetchSnapshotPartFromRoomMember { peer, room, part }]
}

#[cfg(test)]
mod tests {
    use super::retry_fetch;
    use crate::room_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        assert_eq!(retry_fetch(&mut state), Vec::new());
    }
}
