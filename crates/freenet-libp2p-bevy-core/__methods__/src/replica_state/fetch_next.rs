use crate::room_sync;

pub fn fetch_next<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    let Some(room) = state.room.clone() else {
        return Vec::new();
    };
    let peer = state
        .member_snapshot
        .members
        .iter()
        .find(|member| !state.member_snapshot.asked_members.contains(member))
        .cloned();
    let Some(peer) = peer else {
        return Vec::new();
    };
    state.member_snapshot.asked_members.push(peer.clone());
    state.member_snapshot.current_source = Some(peer.clone());
    vec![
        room_sync::state_machine::Output::FetchSnapshotPartFromRoomMember {
            peer,
            room,
            part: 0,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::fetch_next;
    use crate::room_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        assert_eq!(fetch_next(&mut state), Vec::new());
    }
}
