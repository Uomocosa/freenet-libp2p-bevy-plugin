use crate::room_sync;

pub fn expire_overdue_replica_slice_requests<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
    now: room_sync::ElapsedSinceAppStart,
) -> Vec<room_sync::state_machine::Output<Op>> {
    let mut expired: usize = 0;
    for awaiting in state.slice_anti_entropy.fetches_awaiting_reply.values_mut() {
        let before = awaiting.len();
        awaiting
            .retain(|_, at| (*now).saturating_sub(**at) < room_sync::REPLICA_SLICE_REPLY_TIMEOUT);
        expired = expired.saturating_add(before.saturating_sub(awaiting.len()));
    }
    state
        .slice_anti_entropy
        .fetches_awaiting_reply
        .retain(|_, awaiting| !awaiting.is_empty());
    if expired > 0 {
        tracing::debug!(target: "room_sync", "replica slice requests expired expired={}", expired);
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::expire_overdue_replica_slice_requests;
    use crate::identity;
    use crate::p2p;
    use crate::room_sync;
    use crate::room_sync::state_machine::{Input, LobbyRoomStatus, ReplicaState};

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let now = room_sync::ElapsedSinceAppStart::default();
        assert_eq!(
            expire_overdue_replica_slice_requests(&mut state, now),
            Vec::new()
        );
    }

    fn at_secs(secs: u64) -> room_sync::ElapsedSinceAppStart {
        room_sync::ElapsedSinceAppStart::from(std::time::Duration::from_secs(secs))
    }

    #[test]
    fn an_overdue_replica_slice_request_is_cleared() {
        let room = identity::RoomName::from("r");
        let mut joiner = ReplicaState::<u8>::default();
        joiner.update(
            Input::Joined {
                room,
                prefix: p2p::HistoryKeyPrefix::from("prefix"),
                members: Vec::new(),
                lobby_status: LobbyRoomStatus::default(),
            },
            at_secs(0),
        );
        joiner.update(Input::SnapshotMissingFromRoomMember, at_secs(0));
        let peer = identity::PeerId::from("a");
        let index = room_sync::ReplicaSliceIndex(2);
        joiner
            .slice_anti_entropy
            .fetches_awaiting_reply
            .entry(peer.clone())
            .or_default()
            .insert(index, at_secs(0));
        let remote = Input::Remote {
            room: identity::RoomName::from("other"),
            meta: room_sync::OpMeta {
                lamport: 1,
                author: 1,
            },
            op: 1u8,
        };
        let outputs = joiner.update(remote, at_secs(13));
        assert_eq!(outputs, Vec::new());
        assert_eq!(
            joiner.slice_anti_entropy.fetches_awaiting_reply.get(&peer),
            None
        );
    }
}
