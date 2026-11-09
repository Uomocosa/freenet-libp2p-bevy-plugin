use crate::room_sync;

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
        return state.finish_member_fetch();
    };
    if state.member_snapshot.total_parts == 0 {
        return state.finish_member_fetch();
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
