use crate::identity;
use crate::p2p;
use crate::room_sync;
use std::collections::BTreeMap;

use atomic_delegate_macros::atomic_delegates;

pub struct ReplicaState<Op> {
    pub room: Option<identity::RoomName>,
    pub phase: room_sync::state_machine::Phase,
    pub buffered_ops: Vec<(room_sync::OpMeta, Op)>,
    pub member_snapshot: room_sync::state_machine::MemberSnapshotFetchState,
    pub slice_anti_entropy: room_sync::state_machine::SliceAntiEntropyState,
    pub kademlia_history: room_sync::state_machine::KademliaHistoryState,
}

impl<Op> Default for ReplicaState<Op> {
    fn default() -> Self {
        let history_defaults = room_sync::HistoryConfig::default();
        Self {
            room: None,
            phase: room_sync::state_machine::Phase::Idle,
            buffered_ops: Vec::new(),
            member_snapshot: room_sync::state_machine::MemberSnapshotFetchState::default(),
            slice_anti_entropy: room_sync::state_machine::SliceAntiEntropyState {
                fetches_awaiting_reply: BTreeMap::new(),
                max_fetches_per_peer: room_sync::AntiEntropyConfig::default()
                    .max_slice_fetches_per_peer,
            },
            kademlia_history: room_sync::state_machine::KademliaHistoryState {
                is_enabled: false,
                prefix: None,
                lobby_status: room_sync::state_machine::LobbyRoomStatus::default(),
                fetch_retries: 0,
                max_fetch_retries: history_defaults.max_fetch_retries,
                per_room_bytes: history_defaults.per_room_bytes,
                record_ttl: history_defaults.record_ttl,
                pending_fetch: None,
            },
        }
    }
}

#[atomic_delegates]
impl<Op> ReplicaState<Op> {
    pub fn fetch_next(&mut self) -> Vec<room_sync::state_machine::Output<Op>> {}
    pub fn handle_failed(
        &mut self,
        from: &identity::PeerId,
    ) -> Vec<room_sync::state_machine::Output<Op>> {
    }
    pub fn handle_fetch(
        &self,
        id: p2p::ReplyId,
        room: identity::RoomName,
        part: u32,
    ) -> Vec<room_sync::state_machine::Output<Op>> {
    }
    pub fn handle_history_record_not_found(
        &mut self,
        id: &p2p::RoomHistoryRecordKind,
    ) -> Vec<room_sync::state_machine::Output<Op>> {
    }
    pub fn handle_joined(
        &mut self,
        room: identity::RoomName,
        prefix: p2p::HistoryKeyPrefix,
        members: Vec<identity::PeerId>,
        lobby_status: room_sync::state_machine::LobbyRoomStatus,
    ) -> Vec<room_sync::state_machine::Output<Op>> {
    }
    pub fn handle_members(
        &mut self,
        members: Vec<identity::PeerId>,
        prefix: p2p::HistoryKeyPrefix,
        lobby_status: room_sync::state_machine::LobbyRoomStatus,
    ) -> Vec<room_sync::state_machine::Output<Op>> {
    }
    pub fn handle_remote(
        &mut self,
        room: &identity::RoomName,
        meta: room_sync::OpMeta,
        op: Op,
    ) -> Vec<room_sync::state_machine::Output<Op>> {
    }
    pub fn handle_remote_slice_content_hashes(
        &mut self,
        from: &identity::PeerId,
        local: &room_sync::ReplicaSliceContentHashes,
        remote: &room_sync::ReplicaSliceContentHashes,
    ) -> Vec<room_sync::state_machine::Output<Op>> {
    }
    pub fn handle_replica_slice_received(
        &mut self,
        from: &identity::PeerId,
        room: &identity::RoomName,
        index: room_sync::ReplicaSliceIndex,
        data: Vec<u8>,
    ) -> Vec<room_sync::state_machine::Output<Op>> {
    }
    pub fn handle_replica_slice_requested(
        &self,
        id: p2p::ReplyId,
        room: identity::RoomName,
        index: room_sync::ReplicaSliceIndex,
    ) -> Vec<room_sync::state_machine::Output<Op>> {
    }
    pub fn handle_snapshot(
        &mut self,
        room: identity::RoomName,
        part: u32,
        total: u32,
        data: Vec<u8>,
    ) -> Vec<room_sync::state_machine::Output<Op>> {
    }
    pub fn publish_slice_content_hashes(
        &self,
        local: room_sync::ReplicaSliceContentHashes,
    ) -> Vec<room_sync::state_machine::Output<Op>> {
    }
    pub fn retry_fetch(&mut self) -> Vec<room_sync::state_machine::Output<Op>> {}
    pub fn start_history_write_check(&mut self) -> Vec<room_sync::state_machine::Output<Op>> {}
    pub fn start_kademlia_history_fetch(&mut self) -> Vec<room_sync::state_machine::Output<Op>> {}
    pub fn tick(&mut self) -> Vec<room_sync::state_machine::Output<Op>> {}
}

#[rustfmt::skip]
impl<Op> ReplicaState<Op> {
    pub fn handle_history_record_found(
        &mut self,
        id: &p2p::RoomHistoryRecordKind,
        value: Vec<u8>,
    ) -> Vec<room_sync::state_machine::Output<Op>> {
        match id {
            p2p::RoomHistoryRecordKind::Manifest(prefix) => {
                let Some(pending) = self.kademlia_history.pending_fetch.as_mut() else {
                    return Vec::new();
                };
                if &pending.prefix != prefix || pending.manifest.is_some() {
                    return Vec::new();
                }
                let Ok(manifest) = bincode::deserialize::<p2p::HistoryManifest>(&value) else {
                    return self.handle_history_record_not_found(id);
                };
                if manifest.chunk_count == 0 {
                    return self.handle_history_record_not_found(id);
                }
                let outputs: Vec<_> = (0..manifest.chunk_count)
                    .map(|index| room_sync::state_machine::Output::GetHistoryRecord {
                        id: p2p::RoomHistoryRecordKind::Chunk(p2p::HistoryChunkId {
                            prefix: prefix.clone(),
                            content_hash: manifest.content_hash.clone(),
                            chunk: p2p::ChunkIndex::from(index),
                        }),
                    })
                    .collect();
                pending.manifest = Some(manifest);
                pending.received_chunks.clear();
                outputs
            }
            p2p::RoomHistoryRecordKind::Chunk(chunk_id) => {
                let Some(pending) = self.kademlia_history.pending_fetch.as_mut() else {
                    return Vec::new();
                };
                let Some(manifest) = pending.manifest.clone() else {
                    return Vec::new();
                };
                if pending.prefix != chunk_id.prefix
                    || manifest.content_hash != chunk_id.content_hash
                    || *chunk_id.chunk >= u64::from(manifest.chunk_count)
                {
                    return Vec::new();
                }
                pending.received_chunks.insert(chunk_id.chunk, value);
                if u32::try_from(pending.received_chunks.len()).unwrap_or(0) < manifest.chunk_count {
                    return Vec::new();
                }
                let prefix = pending.prefix.clone();
                let ordered_chunks: Vec<Vec<u8>> = pending.received_chunks.values().cloned().collect();
                let Ok(bytes) = p2p::ChunkedSnapshot::join(&ordered_chunks, &manifest) else {
                    return self.handle_history_record_not_found(id);
                };
                self.kademlia_history.pending_fetch = None;
                complete_history_fetch(self, prefix, manifest, bytes)
            }
        }
    }
    pub fn update(
        &mut self,
        input: room_sync::state_machine::Input<Op>,
    ) -> Vec<room_sync::state_machine::Output<Op>> {
        match input {
            room_sync::state_machine::Input::Joined {
                room,
                prefix,
                members,
                lobby_status,
            } => self.handle_joined(room, prefix, members, lobby_status),
            room_sync::state_machine::Input::Members {
                members,
                prefix,
                lobby_status,
            } => self.handle_members(members, prefix, lobby_status),
            room_sync::state_machine::Input::Remote { room, meta, op } => {
                self.handle_remote(&room, meta, op)
            }
            room_sync::state_machine::Input::SnapshotPartReceivedFromRoomMember {
                room,
                part,
                total,
                data,
            } => self.handle_snapshot(room, part, total, data),
            room_sync::state_machine::Input::SnapshotPartRequestedByRoomMember { id, room, part } => {
                self.handle_fetch(id, room, part)
            }
            room_sync::state_machine::Input::Tick => self.tick(),
            room_sync::state_machine::Input::RequestToRoomMemberFailed { from } => {
                self.handle_failed(&from)
            }
            room_sync::state_machine::Input::HistoryWriteTimerFired => {
                self.start_history_write_check()
            }
            room_sync::state_machine::Input::HistoryRecordFound { id, value } => {
                self.handle_history_record_found(&id, value)
            }
            room_sync::state_machine::Input::HistoryRecordNotFound { id } => {
                self.handle_history_record_not_found(&id)
            }
            room_sync::state_machine::Input::AntiEntropyTimerFired { local } => {
                self.publish_slice_content_hashes(local)
            }
            room_sync::state_machine::Input::RemoteSliceContentHashes {
                from,
                local,
                remote,
            } => self.handle_remote_slice_content_hashes(&from, &local, &remote,
            ),
            room_sync::state_machine::Input::ReplicaSliceRequested { id, room, index } => {
                self.handle_replica_slice_requested(id, room, index)
            }
            room_sync::state_machine::Input::ReplicaSliceReceived {
                from,
                room,
                index,
                data,
            } => self.handle_replica_slice_received(&from, &room, index, data,
            ),
        }
    }
}

// needed helper: emits the phase-appropriate output once all chunks of a Kademlia snapshot are verified
fn complete_history_fetch<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
    prefix: p2p::HistoryKeyPrefix,
    manifest: p2p::HistoryManifest,
    bytes: Vec<u8>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    match state.phase {
        room_sync::state_machine::Phase::FetchingHistoryFromKademlia => {
            state.phase = room_sync::state_machine::Phase::Ready;
            let mut outputs = vec![room_sync::state_machine::Output::Merge { bytes }];
            for (meta, op) in state.buffered_ops.drain(..) {
                outputs.push(room_sync::state_machine::Output::Apply { meta, op });
            }
            outputs
        }
        room_sync::state_machine::Phase::Ready => {
            vec![room_sync::state_machine::Output::MergeAndWriteHistory {
                prefix,
                stored_manifest: Some(manifest),
                stored_snapshot: Some(bytes),
            }]
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {

    use crate::identity;
    use crate::p2p;
    use crate::room_sync;
    use crate::room_sync::state_machine::{
        Input, LobbyRoomStatus, Output, PendingHistoryFetch, Phase, ReplicaState,
    };
    use std::collections::BTreeMap;

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let prefix = p2p::HistoryKeyPrefix::from("abc");
        state.room = Some(identity::RoomName::from("r"));
        state.phase = Phase::FetchingHistoryFromKademlia;
        state.kademlia_history.is_enabled = true;
        state.kademlia_history.prefix = Some(prefix.clone());
        state.kademlia_history.pending_fetch = Some(PendingHistoryFetch {
            prefix: prefix.clone(),
            manifest: None,
            received_chunks: BTreeMap::new(),
        });
        let split = p2p::ChunkedSnapshot::split(&[1, 2, 3], 4).expect("split");
        let manifest_bytes = bincode::serialize(&split.manifest).expect("manifest");
        let manifest_out = state.handle_history_record_found(
            &p2p::RoomHistoryRecordKind::Manifest(prefix.clone()),
            manifest_bytes,
        );
        let chunk_kind = p2p::RoomHistoryRecordKind::Chunk(p2p::HistoryChunkId {
            prefix,
            content_hash: split.manifest.content_hash,
            chunk: p2p::ChunkIndex(0),
        });
        assert_eq!(
            manifest_out,
            vec![Output::GetHistoryRecord {
                id: chunk_kind.clone(),
            }]
        );
        let chunk_out = state.handle_history_record_found(&chunk_kind, split.chunks[0].clone());
        assert_eq!(state.phase, Phase::Ready);
        assert_eq!(
            chunk_out,
            vec![Output::Merge {
                bytes: vec![1, 2, 3]
            }]
        );
    }

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
    fn test_usage_1() {
        let mut state = ReplicaState::<u8>::default();
        let room = identity::RoomName::from("r");
        let outputs = state.update(Input::Joined {
            room: room.clone(),
            prefix: prefix(),
            members: vec![identity::PeerId::from("a")],
            lobby_status: alive_status(),
        });
        assert_eq!(state.phase, Phase::FetchingSnapshotFromRoomMember);
        assert_eq!(
            outputs,
            vec![Output::FetchSnapshotPartFromRoomMember {
                peer: identity::PeerId::from("a"),
                room: room.clone(),
                part: 0,
            }]
        );
        state.update(Input::SnapshotPartReceivedFromRoomMember {
            room: room.clone(),
            part: 0,
            total: 1,
            data: vec![1, 2],
        });
        assert_eq!(state.phase, Phase::Ready);
        let outputs = state.update(Input::Remote {
            room,
            meta: meta(1),
            op: 9u8,
        });
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
        let mut state = ReplicaState::<u8>::default();
        let room = identity::RoomName::from("r");
        let peer = identity::PeerId::from("a");
        state.update(Input::Joined {
            room: room.clone(),
            prefix: prefix(),
            members: vec![peer.clone()],
            lobby_status: alive_status(),
        });
        state.update(Input::SnapshotPartReceivedFromRoomMember {
            room: room.clone(),
            part: 0,
            total: 2,
            data: vec![1],
        });
        let outputs = state.update(Input::RequestToRoomMemberFailed {
            from: identity::PeerId::from("a"),
        });
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
        let mut joiner = ReplicaState::<u8>::default();
        let room = identity::RoomName::from("r");
        let peer = identity::PeerId::from("a");
        let outputs = joiner.update(Input::Joined {
            room: room.clone(),
            prefix: prefix(),
            members: vec![peer.clone()],
            lobby_status: alive_status(),
        });
        assert_eq!(
            outputs,
            vec![Output::FetchSnapshotPartFromRoomMember {
                peer,
                room: room.clone(),
                part: 0,
            }]
        );
        joiner.update(Input::Remote {
            room: room.clone(),
            meta: meta(4),
            op: 3u8,
        });
        assert_eq!(joiner.buffered_ops.len(), 1);
        let snapshot = vec![1u8, 2, 3];
        let outputs = joiner.update(Input::SnapshotPartReceivedFromRoomMember {
            room,
            part: 0,
            total: 1,
            data: snapshot.clone(),
        });
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
        let mut state = ReplicaState::<u8>::default();
        let room = identity::RoomName::from("r");
        state.update(Input::Joined {
            room: room.clone(),
            prefix: prefix(),
            members: vec![identity::PeerId::from("a")],
            lobby_status: alive_status(),
        });
        let outputs = state.update(Input::Remote {
            room: room.clone(),
            meta: meta(1),
            op: 5u8,
        });
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.buffered_ops.len(), 1);
        let outputs = state.update(Input::SnapshotPartReceivedFromRoomMember {
            room,
            part: 0,
            total: 1,
            data: vec![0],
        });
        assert!(matches!(
            outputs.as_slice(),
            [Output::Merge { .. }, Output::Apply { op: 5u8, .. }]
        ));
        assert_eq!(state.buffered_ops, Vec::new());
    }

    fn ready_state(room: &identity::RoomName) -> ReplicaState<u8> {
        let mut state = ReplicaState::<u8>::default();
        state.update(Input::Joined {
            room: room.clone(),
            prefix: prefix(),
            members: Vec::new(),
            lobby_status: alive_status(),
        });
        state.update(Input::Tick);
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
        let mut idle = ReplicaState::<u8>::default();
        let local = hashes(&[(0, 1)]);
        let outputs = idle.update(Input::AntiEntropyTimerFired {
            local: local.clone(),
        });
        assert_eq!(outputs, Vec::new());
        let mut ready = ready_state(&room);
        let outputs = ready.update(Input::AntiEntropyTimerFired {
            local: local.clone(),
        });
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
        let first = state.update(Input::RemoteSliceContentHashes {
            from: peer.clone(),
            local: Vec::new(),
            remote: remote.clone(),
        });
        assert_eq!(fetched_indices(&first), vec![0, 1]);
        let again = state.update(Input::RemoteSliceContentHashes {
            from: peer,
            local: Vec::new(),
            remote,
        });
        assert_eq!(fetched_indices(&again), Vec::<u32>::new());
    }

    #[test]
    fn received_slice_is_merged_and_frees_its_fetch() {
        let room = identity::RoomName::from("r");
        let peer = identity::PeerId::from("a");
        let mut state = ready_state(&room);
        let remote = hashes(&[(3, 1)]);
        state.update(Input::RemoteSliceContentHashes {
            from: peer.clone(),
            local: Vec::new(),
            remote: remote.clone(),
        });
        let outputs = state.update(Input::ReplicaSliceReceived {
            from: peer.clone(),
            room,
            index: room_sync::ReplicaSliceIndex(3),
            data: vec![7],
        });
        assert_eq!(outputs, vec![Output::Merge { bytes: vec![7] }]);
        let refetch = state.update(Input::RemoteSliceContentHashes {
            from: peer,
            local: Vec::new(),
            remote,
        });
        assert_eq!(fetched_indices(&refetch), vec![3]);
    }

    #[test]
    fn failure_frees_every_fetch_to_that_peer() {
        let room = identity::RoomName::from("r");
        let peer = identity::PeerId::from("a");
        let mut state = ready_state(&room);
        let remote = hashes(&[(0, 1)]);
        state.update(Input::RemoteSliceContentHashes {
            from: peer.clone(),
            local: Vec::new(),
            remote: remote.clone(),
        });
        state.update(Input::RequestToRoomMemberFailed { from: peer.clone() });
        let refetch = state.update(Input::RemoteSliceContentHashes {
            from: peer,
            local: Vec::new(),
            remote,
        });
        assert_eq!(fetched_indices(&refetch), vec![0]);
    }

    #[test]
    fn slice_hashes_are_ignored_while_fetching_the_join_snapshot() {
        let room = identity::RoomName::from("r");
        let peer = identity::PeerId::from("a");
        let mut state = ReplicaState::<u8>::default();
        state.update(Input::Joined {
            room,
            prefix: prefix(),
            members: vec![peer.clone()],
            lobby_status: alive_status(),
        });
        let outputs = state.update(Input::RemoteSliceContentHashes {
            from: peer,
            local: Vec::new(),
            remote: hashes(&[(0, 1)]),
        });
        assert_eq!(outputs, Vec::new());
    }

    #[test]
    fn history_fallback_retries_then_reports_unavailable_when_room_is_alive() {
        let room = identity::RoomName::from("r");
        let mut state = ReplicaState::<u8>::default();
        state.kademlia_history.is_enabled = true;
        state.kademlia_history.max_fetch_retries = 2;
        let first = state.update(Input::Joined {
            room: room.clone(),
            prefix: prefix(),
            members: Vec::new(),
            lobby_status: alive_status(),
        });
        let manifest = p2p::RoomHistoryRecordKind::Manifest(prefix());
        assert_eq!(
            first,
            vec![Output::GetHistoryRecord {
                id: manifest.clone()
            }]
        );
        for _ in 0..2 {
            let retry = state.update(Input::HistoryRecordNotFound {
                id: manifest.clone(),
            });
            assert_eq!(
                retry,
                vec![Output::GetHistoryRecord {
                    id: manifest.clone()
                }]
            );
        }
        let exhausted = state.update(Input::HistoryRecordNotFound { id: manifest });
        assert_eq!(exhausted, vec![Output::HistoryUnavailable { room }]);
        assert_eq!(state.phase, Phase::Failed);
    }
}
