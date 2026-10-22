use crate::identity;
use crate::room_sync;

pub fn handle_remote<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
    room: &identity::RoomName,
    meta: room_sync::OpMeta,
    op: Op,
) -> Vec<room_sync::state_machine::Output<Op>> {
    if state.room.as_ref() != Some(room) {
        return Vec::new();
    }
    if state.phase == room_sync::state_machine::Phase::Ready {
        vec![room_sync::state_machine::Output::Apply { meta, op }]
    } else {
        state.buffered_ops.push((meta, op));
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::handle_remote;
    use crate::identity;
    use crate::room_sync;
    use crate::room_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let outputs = handle_remote(
            &mut state,
            &identity::RoomName::from("room"),
            room_sync::OpMeta {
                lamport: 1,
                author: 2,
            },
            7_u8,
        );
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.buffered_ops, Vec::new());
    }
}
