use crate::identity;
use crate::room_sync;

pub fn handle_snapshot<Op>(
    state: &mut room_sync::state_machine::State<Op>,
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
    let complete: Vec<Vec<u8>> = state
        .member_snapshot
        .received_parts
        .values()
        .cloned()
        .collect();
    let bytes = room_sync::join_parts(&complete);
    state.phase = room_sync::state_machine::Phase::Ready;
    let mut outputs = vec![room_sync::state_machine::Output::Merge { bytes }];
    for (meta, op) in state.buffered_ops.drain(..) {
        outputs.push(room_sync::state_machine::Output::Apply { meta, op });
    }
    outputs
}

// no test_usage necessary
