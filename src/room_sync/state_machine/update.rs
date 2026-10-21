use crate::room_sync;

pub fn update<Op>(
    state: &mut room_sync::state_machine::State<Op>,
    input: room_sync::state_machine::Input<Op>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    match input {
        room_sync::state_machine::Input::Joined {
            room,
            prefix,
            members,
            lobby_status,
        } => room_sync::state_machine::handle_joined(state, room, prefix, members, lobby_status),
        room_sync::state_machine::Input::Members {
            members,
            prefix,
            lobby_status,
        } => room_sync::state_machine::handle_members(state, members, prefix, lobby_status),
        room_sync::state_machine::Input::Remote { room, meta, op } => {
            room_sync::state_machine::handle_remote(state, &room, meta, op)
        }
        room_sync::state_machine::Input::SnapshotPartReceivedFromRoomMember {
            room,
            part,
            total,
            data,
        } => room_sync::state_machine::handle_snapshot(state, room, part, total, data),
        room_sync::state_machine::Input::SnapshotPartRequestedByRoomMember { id, room, part } => {
            room_sync::state_machine::handle_fetch(state, id, room, part)
        }
        room_sync::state_machine::Input::Tick => room_sync::state_machine::tick(state),
        room_sync::state_machine::Input::RequestToRoomMemberFailed { from } => {
            room_sync::state_machine::handle_failed(state, &from)
        }
        room_sync::state_machine::Input::HistoryWriteTimerFired => {
            room_sync::state_machine::start_history_write_check(state)
        }
        room_sync::state_machine::Input::HistoryRecordFound { id, value } => {
            room_sync::state_machine::handle_history_record_found(state, &id, value)
        }
        room_sync::state_machine::Input::HistoryRecordNotFound { id } => {
            room_sync::state_machine::handle_history_record_not_found(state, &id)
        }
        room_sync::state_machine::Input::AntiEntropyTimerFired { local } => {
            room_sync::state_machine::publish_slice_content_hashes(state, local)
        }
        room_sync::state_machine::Input::RemoteSliceContentHashes {
            from,
            local,
            remote,
        } => room_sync::state_machine::handle_remote_slice_content_hashes(
            state, &from, &local, &remote,
        ),
        room_sync::state_machine::Input::ReplicaSliceRequested { id, room, index } => {
            room_sync::state_machine::handle_replica_slice_requested(state, id, room, index)
        }
        room_sync::state_machine::Input::ReplicaSliceReceived {
            from,
            room,
            index,
            data,
        } => room_sync::state_machine::handle_replica_slice_received(
            state, &from, &room, index, data,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::update;
    use crate::identity;
    use crate::p2p;
    use crate::room_sync;
    use crate::room_sync::state_machine::{Input, LobbyRoomStatus, Output, Phase, State};

    fn meta(lamport: u32) -> room_sync::OpMeta {
        room_sync::OpMeta { lamport, author: 1 }
    }

    fn prefix() -> p2p::HistoryKeyPrefix {
        p2p::HistoryKeyPrefix::from("prefix")
    }

    fn alive_status() -> LobbyRoomStatus {
        LobbyRoomStatus {
            is_room_alive_in_lobby: true,
            has_other_lobby_holders: true,
        }
    }

    #[test]
    fn test_usage() {
        let mut state = State::<u8>::default();
        let room = identity::RoomName::from("r");
        let outputs = update(
            &mut state,
            Input::Joined {
                room: room.clone(),
                prefix: prefix(),
                members: vec![identity::PeerId::from("a")],
                lobby_status: alive_status(),
            },
        );
        assert_eq!(state.phase, Phase::FetchingSnapshotFromRoomMember);
        assert_eq!(
            outputs,
            vec![Output::FetchSnapshotPartFromRoomMember {
                peer: identity::PeerId::from("a"),
                room: room.clone(),
                part: 0,
            }]
        );
        update(
            &mut state,
            Input::SnapshotPartReceivedFromRoomMember {
                room: room.clone(),
                part: 0,
                total: 1,
                data: vec![1, 2],
            },
        );
        assert_eq!(state.phase, Phase::Ready);
        let outputs = update(
            &mut state,
            Input::Remote {
                room,
                meta: meta(1),
                op: 9u8,
            },
        );
        assert_eq!(
            outputs,
            vec![Output::Apply {
                meta: meta(1),
                op: 9u8,
            }]
        );
    }

    #[test]
    fn retries_the_same_source_for_the_missing_part() {
        let mut state = State::<u8>::default();
        let room = identity::RoomName::from("r");
        let peer = identity::PeerId::from("a");
        update(
            &mut state,
            Input::Joined {
                room: room.clone(),
                prefix: prefix(),
                members: vec![peer.clone()],
                lobby_status: alive_status(),
            },
        );
        update(
            &mut state,
            Input::SnapshotPartReceivedFromRoomMember {
                room: room.clone(),
                part: 0,
                total: 2,
                data: vec![1],
            },
        );
        let outputs = update(
            &mut state,
            Input::RequestToRoomMemberFailed {
                from: identity::PeerId::from("a"),
            },
        );
        assert_eq!(
            outputs,
            vec![Output::FetchSnapshotPartFromRoomMember {
                peer,
                room,
                part: 1,
            }]
        );
    }

    #[test]
    fn late_joiner_fetches_then_applies_buffered_ops() {
        let mut joiner = State::<u8>::default();
        let room = identity::RoomName::from("r");
        let peer = identity::PeerId::from("a");
        let outputs = update(
            &mut joiner,
            Input::Joined {
                room: room.clone(),
                prefix: prefix(),
                members: vec![peer.clone()],
                lobby_status: alive_status(),
            },
        );
        assert_eq!(
            outputs,
            vec![Output::FetchSnapshotPartFromRoomMember {
                peer,
                room: room.clone(),
                part: 0,
            }]
        );
        update(
            &mut joiner,
            Input::Remote {
                room: room.clone(),
                meta: meta(4),
                op: 3u8,
            },
        );
        assert_eq!(joiner.buffered_ops.len(), 1);
        let snapshot = vec![1u8, 2, 3];
        let outputs = update(
            &mut joiner,
            Input::SnapshotPartReceivedFromRoomMember {
                room,
                part: 0,
                total: 1,
                data: snapshot.clone(),
            },
        );
        assert_eq!(
            outputs,
            vec![
                Output::Merge { bytes: snapshot },
                Output::Apply {
                    meta: meta(4),
                    op: 3u8,
                },
            ]
        );
        assert_eq!(joiner.phase, Phase::Ready);
    }

    #[test]
    fn buffers_ops_until_the_snapshot_arrives() {
        let mut state = State::<u8>::default();
        let room = identity::RoomName::from("r");
        update(
            &mut state,
            Input::Joined {
                room: room.clone(),
                prefix: prefix(),
                members: vec![identity::PeerId::from("a")],
                lobby_status: alive_status(),
            },
        );
        let outputs = update(
            &mut state,
            Input::Remote {
                room: room.clone(),
                meta: meta(1),
                op: 5u8,
            },
        );
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.buffered_ops.len(), 1);
        let outputs = update(
            &mut state,
            Input::SnapshotPartReceivedFromRoomMember {
                room,
                part: 0,
                total: 1,
                data: vec![0],
            },
        );
        assert!(matches!(
            outputs.as_slice(),
            [Output::Merge { .. }, Output::Apply { op: 5u8, .. }]
        ));
        assert_eq!(state.buffered_ops, Vec::new());
    }

    fn ready_state(room: &identity::RoomName) -> State<u8> {
        let mut state = State::<u8>::default();
        update(
            &mut state,
            Input::Joined {
                room: room.clone(),
                prefix: prefix(),
                members: Vec::new(),
                lobby_status: alive_status(),
            },
        );
        update(&mut state, Input::Tick);
        assert_eq!(state.phase, Phase::Ready);
        state
    }

    fn hashes(entries: &[(u32, u64)]) -> room_sync::ReplicaSliceContentHashes {
        entries
            .iter()
            .map(|(index, hash)| {
                (
                    room_sync::ReplicaSliceIndex(*index),
                    room_sync::ReplicaSliceContentHash(*hash),
                )
            })
            .collect()
    }

    fn fetched_indices(outputs: &[Output<u8>]) -> Vec<u32> {
        outputs
            .iter()
            .filter_map(|output| match output {
                Output::FetchReplicaSlice { index, .. } => Some(**index),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn timer_publishes_slice_hashes_only_when_ready() {
        let room = identity::RoomName::from("r");
        let mut idle = State::<u8>::default();
        let local = hashes(&[(0, 1)]);
        let outputs = update(
            &mut idle,
            Input::AntiEntropyTimerFired {
                local: local.clone(),
            },
        );
        assert_eq!(outputs, Vec::new());
        let mut ready = ready_state(&room);
        let outputs = update(
            &mut ready,
            Input::AntiEntropyTimerFired {
                local: local.clone(),
            },
        );
        assert_eq!(
            outputs,
            vec![Output::PublishSliceContentHashes {
                room,
                hashes: local,
            }]
        );
    }

    #[test]
    fn mismatching_slices_are_fetched_once_and_capped_per_peer() {
        let room = identity::RoomName::from("r");
        let peer = identity::PeerId::from("a");
        let mut state = ready_state(&room);
        state.slice_anti_entropy.max_fetches_per_peer = 2;
        let remote = hashes(&[(0, 1), (1, 1), (2, 1)]);
        let first = update(
            &mut state,
            Input::RemoteSliceContentHashes {
                from: peer.clone(),
                local: Vec::new(),
                remote: remote.clone(),
            },
        );
        assert_eq!(fetched_indices(&first), vec![0, 1]);
        let again = update(
            &mut state,
            Input::RemoteSliceContentHashes {
                from: peer,
                local: Vec::new(),
                remote,
            },
        );
        assert_eq!(fetched_indices(&again), Vec::<u32>::new());
    }

    #[test]
    fn received_slice_is_merged_and_frees_its_fetch() {
        let room = identity::RoomName::from("r");
        let peer = identity::PeerId::from("a");
        let mut state = ready_state(&room);
        let remote = hashes(&[(3, 1)]);
        update(
            &mut state,
            Input::RemoteSliceContentHashes {
                from: peer.clone(),
                local: Vec::new(),
                remote: remote.clone(),
            },
        );
        let outputs = update(
            &mut state,
            Input::ReplicaSliceReceived {
                from: peer.clone(),
                room,
                index: room_sync::ReplicaSliceIndex(3),
                data: vec![7],
            },
        );
        assert_eq!(outputs, vec![Output::Merge { bytes: vec![7] }]);
        let refetch = update(
            &mut state,
            Input::RemoteSliceContentHashes {
                from: peer,
                local: Vec::new(),
                remote,
            },
        );
        assert_eq!(fetched_indices(&refetch), vec![3]);
    }

    #[test]
    fn failure_frees_every_fetch_to_that_peer() {
        let room = identity::RoomName::from("r");
        let peer = identity::PeerId::from("a");
        let mut state = ready_state(&room);
        let remote = hashes(&[(0, 1)]);
        update(
            &mut state,
            Input::RemoteSliceContentHashes {
                from: peer.clone(),
                local: Vec::new(),
                remote: remote.clone(),
            },
        );
        update(
            &mut state,
            Input::RequestToRoomMemberFailed { from: peer.clone() },
        );
        let refetch = update(
            &mut state,
            Input::RemoteSliceContentHashes {
                from: peer,
                local: Vec::new(),
                remote,
            },
        );
        assert_eq!(fetched_indices(&refetch), vec![0]);
    }

    #[test]
    fn slice_hashes_are_ignored_while_fetching_the_join_snapshot() {
        let room = identity::RoomName::from("r");
        let peer = identity::PeerId::from("a");
        let mut state = State::<u8>::default();
        update(
            &mut state,
            Input::Joined {
                room,
                prefix: prefix(),
                members: vec![peer.clone()],
                lobby_status: alive_status(),
            },
        );
        let outputs = update(
            &mut state,
            Input::RemoteSliceContentHashes {
                from: peer,
                local: Vec::new(),
                remote: hashes(&[(0, 1)]),
            },
        );
        assert_eq!(outputs, Vec::new());
    }

    #[test]
    fn history_fallback_retries_then_reports_unavailable_when_room_is_alive() {
        let room = identity::RoomName::from("r");
        let mut state = State::<u8>::default();
        state.kademlia_history.is_enabled = true;
        state.kademlia_history.max_fetch_retries = 2;
        let first = update(
            &mut state,
            Input::Joined {
                room: room.clone(),
                prefix: prefix(),
                members: Vec::new(),
                lobby_status: alive_status(),
            },
        );
        let manifest = p2p::RoomHistoryRecordKind::Manifest(prefix());
        assert_eq!(
            first,
            vec![Output::GetHistoryRecord {
                id: manifest.clone()
            }]
        );
        for _ in 0..2 {
            let retry = update(
                &mut state,
                Input::HistoryRecordNotFound {
                    id: manifest.clone(),
                },
            );
            assert_eq!(
                retry,
                vec![Output::GetHistoryRecord {
                    id: manifest.clone()
                }]
            );
        }
        let exhausted = update(&mut state, Input::HistoryRecordNotFound { id: manifest });
        assert_eq!(exhausted, vec![Output::HistoryUnavailable { room }]);
        assert_eq!(state.phase, Phase::Failed);
    }
}
