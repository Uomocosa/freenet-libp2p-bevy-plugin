use crate::identity;
use crate::room_sync;

pub fn handle_snapshot<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
    room: identity::RoomName,
    part: u32,
    total: u32,
    data: Vec<u8>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    if state.phase != room_sync::state_machine::Phase::FetchingSnapshotFromRoomMember
        || state.room.as_ref() != Some(&room)
        || total == 0
        || part >= total
    {
        return Vec::new();
    }
    if state.member_snapshot.total_parts != total {
        state.member_snapshot.total_parts = total;
        state.member_snapshot.received_parts.clear();
    }
    state.member_snapshot.received_parts.insert(part, data);
    if let Some(next_part) =
        (0..total).find(|index| !state.member_snapshot.received_parts.contains_key(index))
    {
        let Some(peer) = state.member_snapshot.current_source.clone() else {
            return Vec::new();
        };
        return vec![
            room_sync::state_machine::Output::FetchSnapshotPartFromRoomMember {
                peer,
                room,
                part: next_part,
            },
        ];
    }
    let bytes = state.member_snapshot.join_received_parts();
    state.phase = room_sync::state_machine::Phase::Ready;
    let mut outputs = vec![room_sync::state_machine::Output::Merge { bytes }];
    for (meta, op) in state.buffered_ops.drain(..) {
        outputs.push(room_sync::state_machine::Output::Apply { meta, op });
    }
    outputs
}

#[cfg(test)]
mod tests {
    use super::handle_snapshot;
    use crate::identity;
    use crate::room_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let outputs = handle_snapshot(
            &mut state,
            identity::RoomName::from("room"),
            0,
            1,
            vec![1, 2, 3],
        );
        assert_eq!(outputs, Vec::new());
    }
}
