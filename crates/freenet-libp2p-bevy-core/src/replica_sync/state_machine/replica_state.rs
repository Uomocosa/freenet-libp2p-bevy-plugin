use crate::identity;
use crate::p2p;
use crate::replica_sync;
use std::collections::BTreeMap;

use atomic_delegate_macros::atomic_delegates;

pub struct ReplicaState<Op> {
    pub zone: Option<identity::ZoneName>,
    pub phase: replica_sync::state_machine::Phase,
    pub buffered_ops: Vec<(replica_sync::OpMeta, Op)>,
    pub member_snapshot: replica_sync::state_machine::MemberSnapshotFetchState,
    pub slice_anti_entropy: replica_sync::state_machine::SliceAntiEntropyState,
    pub kademlia_history: replica_sync::state_machine::KademliaHistoryState,
}

impl<Op> Default for ReplicaState<Op> {
    fn default() -> Self {
        let history_defaults = replica_sync::HistoryConfig::default();
        Self {
            zone: None,
            phase: replica_sync::state_machine::Phase::Idle,
            buffered_ops: Vec::new(),
            member_snapshot: replica_sync::state_machine::MemberSnapshotFetchState::default(),
            slice_anti_entropy: replica_sync::state_machine::SliceAntiEntropyState {
                fetches_awaiting_reply: BTreeMap::new(),
                max_fetches_per_peer: replica_sync::AntiEntropyConfig::default()
                    .max_slice_fetches_per_peer,
            },
            kademlia_history: replica_sync::state_machine::KademliaHistoryState {
                is_enabled: false,
                prefix: None,
                liveness: replica_sync::state_machine::ZoneLiveness::default(),
                fetch_retries: 0,
                max_fetch_retries: history_defaults.max_fetch_retries,
                per_zone_bytes: history_defaults.per_zone_bytes,
                record_ttl: history_defaults.record_ttl,
                pending_fetch: None,
            },
        }
    }
}

#[atomic_delegates]
impl<Op> ReplicaState<Op> {
    pub fn fetch_next(&mut self) -> Vec<replica_sync::state_machine::Output<Op>> {}
    pub fn finish_member_fetch(&mut self) -> Vec<replica_sync::state_machine::Output<Op>> {}
    pub fn handle_fetch(
        &self,
        id: p2p::ReplyId,
        zone: identity::ZoneName,
        part: u32,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
    pub fn handle_history_record_not_found(
        &mut self,
        id: &p2p::ZoneHistoryRecordKind,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
    pub fn handle_joined(
        &mut self,
        zone: identity::ZoneName,
        prefix: p2p::HistoryKeyPrefix,
        members: Vec<identity::PeerId>,
        liveness: replica_sync::state_machine::ZoneLiveness,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
    pub fn handle_members(
        &mut self,
        members: Vec<identity::PeerId>,
        prefix: p2p::HistoryKeyPrefix,
        liveness: replica_sync::state_machine::ZoneLiveness,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
    pub fn handle_remote(
        &mut self,
        zone: &identity::ZoneName,
        meta: replica_sync::OpMeta,
        op: Op,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
    pub fn handle_remote_slice_content_hashes(
        &self,
        from: &identity::PeerId,
        local: &replica_sync::ReplicaSliceContentHashes,
        remote: &replica_sync::ReplicaSliceContentHashes,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
    pub fn handle_replica_slice_received(
        &mut self,
        from: &identity::PeerId,
        zone: &identity::ZoneName,
        index: replica_sync::ReplicaSliceIndex,
        data: Vec<u8>,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
    pub fn handle_replica_slice_request_failed(
        &mut self,
        from: &identity::PeerId,
        index: replica_sync::ReplicaSliceIndex,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
    pub fn handle_replica_slice_requested(
        &self,
        id: p2p::ReplyId,
        zone: identity::ZoneName,
        index: replica_sync::ReplicaSliceIndex,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
    pub fn handle_snapshot(
        &mut self,
        zone: identity::ZoneName,
        part: u32,
        total: u32,
        data: Vec<u8>,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
    pub fn handle_snapshot_missing(&mut self) -> Vec<replica_sync::state_machine::Output<Op>> {}
    pub fn publish_slice_content_hashes(
        &self,
        local: replica_sync::ReplicaSliceContentHashes,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
    pub fn retry_fetch(&mut self) -> Vec<replica_sync::state_machine::Output<Op>> {}
    pub fn start_history_write_check(&mut self) -> Vec<replica_sync::state_machine::Output<Op>> {}
    pub fn start_kademlia_history_fetch(&mut self) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
    pub fn update(
        &mut self,
        input: replica_sync::state_machine::Input<Op>,
        now: replica_sync::ElapsedSinceAppStart,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
    pub fn handle_history_record_found(
        &mut self,
        id: &p2p::ZoneHistoryRecordKind,
        value: Vec<u8>,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
}

impl<Op> ReplicaState<Op> {
    pub(crate) fn dispatch_input(
        &mut self,
        input: replica_sync::state_machine::Input<Op>,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
        match input {
            replica_sync::state_machine::Input::Joined {
                zone,
                prefix,
                members,
                liveness,
            } => self.handle_joined(zone, prefix, members, liveness),
            replica_sync::state_machine::Input::Members {
                members,
                prefix,
                liveness,
            } => self.handle_members(members, prefix, liveness),
            replica_sync::state_machine::Input::Remote { zone, meta, op } => {
                self.handle_remote(&zone, meta, op)
            }
            replica_sync::state_machine::Input::SnapshotPartReceivedFromZoneMember {
                zone,
                part,
                total,
                data,
            } => self.handle_snapshot(zone, part, total, data),
            replica_sync::state_machine::Input::SnapshotPartRequestedByZoneMember {
                id,
                zone,
                part,
            } => self.handle_fetch(id, zone, part),
            replica_sync::state_machine::Input::SnapshotMissingFromZoneMember => {
                self.handle_snapshot_missing()
            }
            replica_sync::state_machine::Input::RequestToZoneMemberFailed { .. } => {
                self.retry_fetch()
            }
            replica_sync::state_machine::Input::HistoryWriteTimerFired => {
                self.start_history_write_check()
            }
            replica_sync::state_machine::Input::HistoryRecordFound { id, value } => {
                self.handle_history_record_found(&id, value)
            }
            replica_sync::state_machine::Input::HistoryRecordNotFound { id } => {
                self.handle_history_record_not_found(&id)
            }
            replica_sync::state_machine::Input::AntiEntropyTimerFired { local } => {
                self.publish_slice_content_hashes(local)
            }
            replica_sync::state_machine::Input::RemoteSliceContentHashes {
                from,
                local,
                remote,
            } => self.handle_remote_slice_content_hashes(&from, &local, &remote),
            replica_sync::state_machine::Input::ReplicaSliceRequested { id, zone, index } => {
                self.handle_replica_slice_requested(id, zone, index)
            }
            replica_sync::state_machine::Input::ReplicaSliceReceived {
                from,
                zone,
                index,
                data,
            } => self.handle_replica_slice_received(&from, &zone, index, data),
            replica_sync::state_machine::Input::ReplicaSliceRequestFailed { from, index } => {
                self.handle_replica_slice_request_failed(&from, index)
            }
        }
    }
    pub(crate) fn expire_overdue_history_record_requests(
        &mut self,
        now: replica_sync::ElapsedSinceAppStart,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
        let Some(prefix) = self
            .kademlia_history
            .pending_fetch
            .as_ref()
            .filter(|pending| {
                pending.records_requested_at.is_some_and(|at| {
                    (*now).saturating_sub(*at) >= replica_sync::HISTORY_RECORD_REPLY_TIMEOUT
                })
            })
            .map(|pending| pending.prefix.clone())
        else {
            return Vec::new();
        };
        tracing::debug!(target: "replica_sync", "history record request expired");
        self.handle_history_record_not_found(&p2p::ZoneHistoryRecordKind::Manifest(prefix))
    }
}

#[atomic_delegates]
impl<Op> ReplicaState<Op> {
    pub(crate) fn expire_overdue_snapshot_part_request(
        &mut self,
        now: replica_sync::ElapsedSinceAppStart,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
    pub(crate) fn expire_overdue_replica_slice_requests(
        &mut self,
        now: replica_sync::ElapsedSinceAppStart,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
    pub(crate) fn expire_overdue_requests(
        &mut self,
        now: replica_sync::ElapsedSinceAppStart,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
    pub(crate) fn complete_history_fetch(
        &mut self,
        prefix: p2p::HistoryKeyPrefix,
        manifest: p2p::HistoryManifest,
        bytes: Vec<u8>,
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
    pub(crate) fn record_request_send_times(
        &mut self,
        outputs: &[replica_sync::state_machine::Output<Op>],
        now: replica_sync::ElapsedSinceAppStart,
    ) {
    }
    pub(crate) fn settle_waiting_phase(
        &mut self,
        outputs_so_far: &[replica_sync::state_machine::Output<Op>],
    ) -> Vec<replica_sync::state_machine::Output<Op>> {
    }
}

#[cfg(test)]
mod tests {

    use crate::identity;
    use crate::p2p;
    use crate::replica_sync;
    use crate::replica_sync::state_machine::{
        Input, Output, PendingHistoryFetch, Phase, ReplicaState, ZoneLiveness,
    };
    use crate::testing;
    use std::collections::BTreeMap;

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let prefix = p2p::HistoryKeyPrefix::from("abc");
        state.zone = Some(identity::ZoneName::from("r"));
        state.phase = Phase::FetchingHistoryFromKademlia;
        state.kademlia_history.is_enabled = true;
        state.kademlia_history.prefix = Some(prefix.clone());
        state.kademlia_history.pending_fetch = Some(PendingHistoryFetch {
            prefix: prefix.clone(),
            manifest: None,
            received_chunks: BTreeMap::new(),
            records_requested_at: None,
        });
        let split = p2p::ChunkedSnapshot::try_from_payload(&[1, 2, 3], 4).expect("split");
        let manifest_bytes = bincode::serialize(&split.manifest).expect("manifest");
        let manifest_out = state.handle_history_record_found(
            &p2p::ZoneHistoryRecordKind::Manifest(prefix.clone()),
            manifest_bytes,
        );
        let chunk_kind = p2p::ZoneHistoryRecordKind::Chunk(p2p::HistoryChunkId {
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

    #[test]
    fn test_usage_1() {
        let mut state = ReplicaState::<u8>::default();
        let zone = identity::ZoneName::from("r");
        let outputs = state.update(
            Input::Joined {
                zone: zone.clone(),
                prefix: p2p::HistoryKeyPrefix::from("prefix"),
                members: vec![identity::PeerId::from("a")],
                liveness: testing::alive_zone_liveness(),
            },
            testing::at_secs(0),
        );
        assert_eq!(state.phase, Phase::FetchingSnapshotFromZoneMember);
        assert_eq!(
            outputs,
            vec![Output::FetchSnapshotPartFromZoneMember {
                peer: identity::PeerId::from("a"),
                zone: zone.clone(),
                part: 0,
            }]
        );
        state.update(
            Input::SnapshotPartReceivedFromZoneMember {
                zone: zone.clone(),
                part: 0,
                total: 1,
                data: vec![1, 2],
            },
            testing::at_secs(0),
        );
        assert_eq!(state.phase, Phase::Ready);
        let outputs = state.update(
            Input::Remote {
                zone,
                meta: testing::meta(1, 1),
                op: 9u8,
            },
            testing::at_secs(0),
        );
        assert_eq!(
            outputs,
            vec![Output::Apply {
                meta: testing::meta(1, 1),
                op: 9u8,
            }]
        );
    }

    #[test]
    fn retries_the_same_source_for_the_missing_part() {
        let mut state = ReplicaState::<u8>::default();
        let zone = identity::ZoneName::from("r");
        let peer = identity::PeerId::from("a");
        state.update(
            Input::Joined {
                zone: zone.clone(),
                prefix: p2p::HistoryKeyPrefix::from("prefix"),
                members: vec![peer.clone()],
                liveness: testing::alive_zone_liveness(),
            },
            testing::at_secs(0),
        );
        state.update(
            Input::SnapshotPartReceivedFromZoneMember {
                zone: zone.clone(),
                part: 0,
                total: 2,
                data: vec![1],
            },
            testing::at_secs(0),
        );
        let outputs = state.update(
            Input::RequestToZoneMemberFailed {
                from: identity::PeerId::from("a"),
            },
            testing::at_secs(0),
        );
        assert_eq!(
            outputs,
            vec![Output::FetchSnapshotPartFromZoneMember {
                peer,
                zone,
                part: 1,
            }]
        );
    }

    #[test]
    fn late_joiner_fetches_then_applies_buffered_ops() {
        let mut joiner = ReplicaState::<u8>::default();
        let zone = identity::ZoneName::from("r");
        let peer = identity::PeerId::from("a");
        let outputs = joiner.update(
            Input::Joined {
                zone: zone.clone(),
                prefix: p2p::HistoryKeyPrefix::from("prefix"),
                members: vec![peer.clone()],
                liveness: testing::alive_zone_liveness(),
            },
            testing::at_secs(0),
        );
        assert_eq!(
            outputs,
            vec![Output::FetchSnapshotPartFromZoneMember {
                peer,
                zone: zone.clone(),
                part: 0,
            }]
        );
        joiner.update(
            Input::Remote {
                zone: zone.clone(),
                meta: testing::meta(4, 1),
                op: 3u8,
            },
            testing::at_secs(0),
        );
        assert_eq!(joiner.buffered_ops.len(), 1);
        let snapshot = vec![1u8, 2, 3];
        let outputs = joiner.update(
            Input::SnapshotPartReceivedFromZoneMember {
                zone,
                part: 0,
                total: 1,
                data: snapshot.clone(),
            },
            testing::at_secs(0),
        );
        assert_eq!(
            outputs,
            vec![
                Output::Merge { bytes: snapshot },
                Output::Apply {
                    meta: testing::meta(4, 1),
                    op: 3u8,
                },
            ]
        );
        assert_eq!(joiner.phase, Phase::Ready);
    }

    #[test]
    fn buffers_ops_until_the_snapshot_arrives() {
        let mut state = ReplicaState::<u8>::default();
        let zone = identity::ZoneName::from("r");
        state.update(
            Input::Joined {
                zone: zone.clone(),
                prefix: p2p::HistoryKeyPrefix::from("prefix"),
                members: vec![identity::PeerId::from("a")],
                liveness: testing::alive_zone_liveness(),
            },
            testing::at_secs(0),
        );
        let outputs = state.update(
            Input::Remote {
                zone: zone.clone(),
                meta: testing::meta(1, 1),
                op: 5u8,
            },
            testing::at_secs(0),
        );
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.buffered_ops.len(), 1);
        let outputs = state.update(
            Input::SnapshotPartReceivedFromZoneMember {
                zone,
                part: 0,
                total: 1,
                data: vec![0],
            },
            testing::at_secs(0),
        );
        assert!(matches!(
            outputs.as_slice(),
            [Output::Merge { .. }, Output::Apply { op: 5u8, .. }]
        ));
        assert_eq!(state.buffered_ops, Vec::new());
    }

    #[test]
    fn timer_publishes_slice_hashes_only_when_ready() {
        let zone = identity::ZoneName::from("r");
        let mut idle = ReplicaState::<u8>::default();
        let local = testing::hashes(&[(0, 1)]);
        let outputs = idle.update(
            Input::AntiEntropyTimerFired {
                local: local.clone(),
            },
            testing::at_secs(0),
        );
        assert_eq!(outputs, Vec::new());
        let mut ready = testing::ready_replica_state(&zone);
        let outputs = ready.update(
            Input::AntiEntropyTimerFired {
                local: local.clone(),
            },
            testing::at_secs(0),
        );
        assert_eq!(
            outputs,
            vec![Output::PublishSliceContentHashes {
                zone,
                hashes: local,
            }]
        );
    }

    #[test]
    fn mismatching_slices_are_fetched_once_and_capped_per_peer() {
        let zone = identity::ZoneName::from("r");
        let peer = identity::PeerId::from("a");
        let mut state = testing::ready_replica_state(&zone);
        state.slice_anti_entropy.max_fetches_per_peer = 2;
        let remote = testing::hashes(&[(0, 1), (1, 1), (2, 1)]);
        let first = state.update(
            Input::RemoteSliceContentHashes {
                from: peer.clone(),
                local: Vec::new(),
                remote: remote.clone(),
            },
            testing::at_secs(0),
        );
        assert_eq!(testing::fetched_replica_slice_indices(&first), vec![0, 1]);
        let again = state.update(
            Input::RemoteSliceContentHashes {
                from: peer,
                local: Vec::new(),
                remote,
            },
            testing::at_secs(0),
        );
        assert_eq!(
            testing::fetched_replica_slice_indices(&again),
            Vec::<u32>::new()
        );
    }

    #[test]
    fn received_slice_is_merged_and_frees_its_fetch() {
        let zone = identity::ZoneName::from("r");
        let peer = identity::PeerId::from("a");
        let mut state = testing::ready_replica_state(&zone);
        let remote = testing::hashes(&[(3, 1)]);
        state.update(
            Input::RemoteSliceContentHashes {
                from: peer.clone(),
                local: Vec::new(),
                remote: remote.clone(),
            },
            testing::at_secs(0),
        );
        let outputs = state.update(
            Input::ReplicaSliceReceived {
                from: peer.clone(),
                zone,
                index: replica_sync::ReplicaSliceIndex(3),
                data: vec![7],
            },
            testing::at_secs(0),
        );
        assert_eq!(outputs, vec![Output::Merge { bytes: vec![7] }]);
        let refetch = state.update(
            Input::RemoteSliceContentHashes {
                from: peer,
                local: Vec::new(),
                remote,
            },
            testing::at_secs(0),
        );
        assert_eq!(testing::fetched_replica_slice_indices(&refetch), vec![3]);
    }

    #[test]
    fn a_replica_slice_failure_frees_only_that_slice() {
        let zone = identity::ZoneName::from("r");
        let peer = identity::PeerId::from("a");
        let mut state = testing::ready_replica_state(&zone);
        state.slice_anti_entropy.max_fetches_per_peer = 2;
        let remote = testing::hashes(&[(0, 1), (1, 1)]);
        state.update(
            Input::RemoteSliceContentHashes {
                from: peer.clone(),
                local: Vec::new(),
                remote: remote.clone(),
            },
            testing::at_secs(0),
        );
        state.update(
            Input::ReplicaSliceRequestFailed {
                from: peer.clone(),
                index: replica_sync::ReplicaSliceIndex(0),
            },
            testing::at_secs(0),
        );
        let refetch = state.update(
            Input::RemoteSliceContentHashes {
                from: peer,
                local: Vec::new(),
                remote,
            },
            testing::at_secs(0),
        );
        assert_eq!(testing::fetched_replica_slice_indices(&refetch), vec![0]);
    }

    #[test]
    fn a_snapshot_failure_keeps_replica_slice_fetches() {
        let zone = identity::ZoneName::from("r");
        let peer = identity::PeerId::from("a");
        let mut state = testing::ready_replica_state(&zone);
        let remote = testing::hashes(&[(0, 1)]);
        state.update(
            Input::RemoteSliceContentHashes {
                from: peer.clone(),
                local: Vec::new(),
                remote: remote.clone(),
            },
            testing::at_secs(0),
        );
        state.update(
            Input::RequestToZoneMemberFailed { from: peer.clone() },
            testing::at_secs(0),
        );
        let refetch = state.update(
            Input::RemoteSliceContentHashes {
                from: peer,
                local: Vec::new(),
                remote,
            },
            testing::at_secs(0),
        );
        assert_eq!(
            testing::fetched_replica_slice_indices(&refetch),
            Vec::<u32>::new()
        );
    }

    #[test]
    fn slice_hashes_are_ignored_while_fetching_the_join_snapshot() {
        let zone = identity::ZoneName::from("r");
        let peer = identity::PeerId::from("a");
        let mut state = ReplicaState::<u8>::default();
        state.update(
            Input::Joined {
                zone,
                prefix: p2p::HistoryKeyPrefix::from("prefix"),
                members: vec![peer.clone()],
                liveness: testing::alive_zone_liveness(),
            },
            testing::at_secs(0),
        );
        let outputs = state.update(
            Input::RemoteSliceContentHashes {
                from: peer,
                local: Vec::new(),
                remote: testing::hashes(&[(0, 1)]),
            },
            testing::at_secs(0),
        );
        assert_eq!(outputs, Vec::new());
    }

    #[test]
    fn history_fallback_retries_then_reports_unavailable_when_zone_is_alive() {
        let zone = identity::ZoneName::from("r");
        let mut state = ReplicaState::<u8>::default();
        state.kademlia_history.is_enabled = true;
        state.kademlia_history.max_fetch_retries = 2;
        let first = state.update(
            Input::Joined {
                zone: zone.clone(),
                prefix: p2p::HistoryKeyPrefix::from("prefix"),
                members: Vec::new(),
                liveness: testing::alive_zone_liveness(),
            },
            testing::at_secs(0),
        );
        let manifest = p2p::ZoneHistoryRecordKind::Manifest(p2p::HistoryKeyPrefix::from("prefix"));
        assert_eq!(
            first,
            vec![Output::GetHistoryRecord {
                id: manifest.clone()
            }]
        );
        for _ in 0..2 {
            let retry = state.update(
                Input::HistoryRecordNotFound {
                    id: manifest.clone(),
                },
                testing::at_secs(0),
            );
            assert_eq!(
                retry,
                vec![Output::GetHistoryRecord {
                    id: manifest.clone()
                }]
            );
        }
        let exhausted = state.update(
            Input::HistoryRecordNotFound { id: manifest },
            testing::at_secs(0),
        );
        assert_eq!(exhausted, vec![Output::HistoryUnavailable { zone }]);
        assert_eq!(state.phase, Phase::Failed);
    }

    #[test]
    fn request_send_times_are_stamped() {
        let mut state = ReplicaState::<u8>::default();
        let zone = identity::ZoneName::from("r");
        let peer = identity::PeerId::from("a");
        let prefix = p2p::HistoryKeyPrefix::from("abc");
        state.zone = Some(zone.clone());
        state.phase = Phase::FetchingSnapshotFromZoneMember;
        state.kademlia_history.pending_fetch = Some(PendingHistoryFetch {
            prefix: prefix.clone(),
            manifest: None,
            received_chunks: BTreeMap::new(),
            records_requested_at: None,
        });
        let send_now = replica_sync::ElapsedSinceAppStart::from(std::time::Duration::from_secs(7));
        let outputs = vec![
            Output::FetchSnapshotPartFromZoneMember {
                peer: peer.clone(),
                zone: zone.clone(),
                part: 0,
            },
            Output::GetHistoryRecord {
                id: p2p::ZoneHistoryRecordKind::Manifest(prefix),
            },
            Output::FetchReplicaSlice {
                peer: peer.clone(),
                zone,
                index: replica_sync::ReplicaSliceIndex(3),
            },
        ];
        state.record_request_send_times(&outputs, send_now);
        assert_eq!(state.member_snapshot.part_requested_at, Some(send_now));
        assert_eq!(
            state
                .kademlia_history
                .pending_fetch
                .as_ref()
                .and_then(|pending| pending.records_requested_at),
            Some(send_now)
        );
        assert_eq!(
            state
                .slice_anti_entropy
                .fetches_awaiting_reply
                .get(&peer)
                .and_then(|awaiting| awaiting.get(&replica_sync::ReplicaSliceIndex(3)).copied()),
            Some(send_now)
        );
    }

    #[test]
    fn settle_leaves_idle_state_alone() {
        let mut state = ReplicaState::<u8>::default();
        assert_eq!(state.settle_waiting_phase(&[]), Vec::new());
    }

    #[test]
    fn no_input_sequence_leaves_a_waiting_phase_without_a_request() {
        let peer = identity::PeerId::from("a");
        let other = identity::PeerId::from("b");
        let failed = Input::RequestToZoneMemberFailed { from: peer.clone() };
        let missing = Input::SnapshotMissingFromZoneMember;
        let sequences: Vec<(bool, Vec<Input<u8>>)> = vec![
            (false, vec![testing::joined_input(Vec::new())]),
            (false, vec![testing::joined_input(vec![peer.clone()])]),
            (
                false,
                vec![
                    testing::joined_input(vec![peer.clone()]),
                    testing::snapshot_part_input(0, 2),
                    failed,
                ],
            ),
            (
                false,
                vec![testing::joined_input(vec![peer.clone(), other]), missing],
            ),
            (
                false,
                vec![
                    testing::joined_input(vec![peer.clone()]),
                    testing::snapshot_part_input(0, 1),
                    testing::snapshot_part_input(0, 1),
                ],
            ),
            (true, vec![testing::joined_input(Vec::new())]),
            (
                true,
                vec![
                    testing::joined_input(Vec::new()),
                    testing::history_record_not_found_input(),
                ],
            ),
            (
                true,
                vec![
                    testing::joined_input(vec![peer]),
                    Input::Members {
                        members: Vec::new(),
                        prefix: p2p::HistoryKeyPrefix::from("prefix"),
                        liveness: ZoneLiveness::default(),
                    },
                ],
            ),
        ];
        for (history_enabled, sequence) in sequences {
            let mut state = ReplicaState::<u8>::default();
            state.kademlia_history.is_enabled = history_enabled;
            for input in sequence {
                state.update(input, replica_sync::ElapsedSinceAppStart::default());
                if state.phase == Phase::FetchingSnapshotFromZoneMember {
                    assert!(
                        state.member_snapshot.part_requested_at.is_some(),
                        "snapshot wait without a request time"
                    );
                }
                if state.phase == Phase::FetchingHistoryFromKademlia {
                    assert!(
                        state.kademlia_history.pending_fetch.is_some(),
                        "history wait without a pending fetch"
                    );
                }
            }
        }
    }

    #[test]
    fn an_overdue_history_record_request_retries_or_fails() {
        let mut state = ReplicaState::<u8>::default();
        let zone = identity::ZoneName::from("r");
        let prefix_value = p2p::HistoryKeyPrefix::from("prefix");
        state.zone = Some(zone);
        state.phase = Phase::FetchingHistoryFromKademlia;
        state.kademlia_history.is_enabled = true;
        state.kademlia_history.prefix = Some(prefix_value.clone());
        state.kademlia_history.liveness = testing::alive_zone_liveness();
        state.kademlia_history.fetch_retries = 0;
        state.kademlia_history.pending_fetch = Some(PendingHistoryFetch {
            prefix: prefix_value.clone(),
            manifest: None,
            received_chunks: BTreeMap::new(),
            records_requested_at: Some(testing::at_secs(0)),
        });
        let outputs = state.update(testing::other_zone_remote_input(1u8), testing::at_secs(71));
        assert_eq!(
            outputs,
            vec![Output::GetHistoryRecord {
                id: p2p::ZoneHistoryRecordKind::Manifest(prefix_value),
            }]
        );
    }
}
