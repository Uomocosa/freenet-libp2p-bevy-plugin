use crate::room_sync;

pub fn expire_overdue_snapshot_part_request<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
    now: room_sync::ElapsedSinceAppStart,
) -> Vec<room_sync::state_machine::Output<Op>> {
    let Some(at) = state.member_snapshot.part_requested_at else {
        return Vec::new();
    };
    if (*now).saturating_sub(*at) < room_sync::SNAPSHOT_PART_REPLY_TIMEOUT {
        return Vec::new();
    }
    state.member_snapshot.part_requested_at = None;
    tracing::debug!(target: "room_sync", "snapshot part request expired");
    state.retry_fetch()
}

#[cfg(test)]
mod tests {
    use super::expire_overdue_snapshot_part_request;
    use crate::identity;
    use crate::p2p;
    use crate::room_sync;
    use crate::room_sync::state_machine::{Input, LobbyRoomStatus, Output, ReplicaState};

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let now = room_sync::ElapsedSinceAppStart::default();
        assert_eq!(
            expire_overdue_snapshot_part_request(&mut state, now),
            Vec::new()
        );
    }

    fn at_secs(secs: u64) -> room_sync::ElapsedSinceAppStart {
        room_sync::ElapsedSinceAppStart::from(std::time::Duration::from_secs(secs))
    }

    fn alive_status() -> LobbyRoomStatus {
        LobbyRoomStatus {
            is_room_alive_in_lobby: true,
            has_other_lobby_holders: true,
        }
    }

    fn other_room_remote() -> Input<u8> {
        Input::Remote {
            room: identity::RoomName::from("other"),
            meta: room_sync::OpMeta {
                lamport: 1,
                author: 1,
            },
            op: 1u8,
        }
    }

    #[test]
    fn an_overdue_snapshot_part_request_is_retried() {
        let mut state = ReplicaState::<u8>::default();
        let room = identity::RoomName::from("r");
        let first = identity::PeerId::from("a");
        let second = identity::PeerId::from("b");
        let joined = state.update(
            Input::Joined {
                room: room.clone(),
                prefix: p2p::HistoryKeyPrefix::from("prefix"),
                members: vec![first.clone(), second.clone()],
                lobby_status: alive_status(),
            },
            at_secs(0),
        );
        assert_eq!(
            joined,
            vec![Output::FetchSnapshotPartFromRoomMember {
                peer: first,
                room: room.clone(),
                part: 0,
            }]
        );
        let outputs = state.update(other_room_remote(), at_secs(13));
        assert_eq!(
            outputs,
            vec![Output::FetchSnapshotPartFromRoomMember {
                peer: second,
                room,
                part: 0,
            }]
        );
        assert_eq!(state.member_snapshot.part_requested_at, Some(at_secs(13)));
    }

    #[test]
    fn a_fresh_snapshot_part_request_is_left_alone() {
        let mut state = ReplicaState::<u8>::default();
        let room = identity::RoomName::from("r");
        state.update(
            Input::Joined {
                room,
                prefix: p2p::HistoryKeyPrefix::from("prefix"),
                members: vec![identity::PeerId::from("a"), identity::PeerId::from("b")],
                lobby_status: alive_status(),
            },
            at_secs(0),
        );
        let outputs = state.update(other_room_remote(), at_secs(5));
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.member_snapshot.part_requested_at, Some(at_secs(0)));
    }
}
