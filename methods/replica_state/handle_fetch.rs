use crate::identity;
use crate::p2p;
use crate::room_sync;

#[must_use]
pub fn handle_fetch<Op>(
    state: &room_sync::state_machine::ReplicaState<Op>,
    id: p2p::ReplyId,
    room: identity::RoomName,
    part: u32,
) -> Vec<room_sync::state_machine::Output<Op>> {
    if state.room.is_none() {
        return Vec::new();
    }
    vec![room_sync::state_machine::Output::SendSnapshotPartToRoomMember { id, room, part }]
}

#[cfg(test)]
mod tests {
    use super::handle_fetch;
    use crate::identity;
    use crate::p2p;
    use crate::room_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let state = ReplicaState::<u8>::default();
        let outputs = handle_fetch(&state, p2p::ReplyId(1), identity::RoomName::from("room"), 0);
        assert_eq!(outputs, Vec::new());
    }
}
