use crate::identity;
use crate::p2p;
use crate::replica_sync;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Idle,
    FetchingSnapshotFromZoneMember,
    FetchingHistoryFromKademlia,
    Ready,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input<Op> {
    Joined {
        zone: identity::ZoneName,
        prefix: p2p::HistoryKeyPrefix,
        members: Vec<identity::PeerId>,
        liveness: replica_sync::state_machine::ZoneLiveness,
    },
    Members {
        members: Vec<identity::PeerId>,
        prefix: p2p::HistoryKeyPrefix,
        liveness: replica_sync::state_machine::ZoneLiveness,
    },
    Remote {
        zone: identity::ZoneName,
        meta: replica_sync::OpMeta,
        op: Op,
    },
    SnapshotPartReceivedFromZoneMember {
        zone: identity::ZoneName,
        part: u32,
        total: u32,
        data: Vec<u8>,
    },
    SnapshotPartRequestedByZoneMember {
        id: p2p::ReplyId,
        zone: identity::ZoneName,
        part: u32,
    },
    SnapshotMissingFromZoneMember,
    RequestToZoneMemberFailed {
        from: identity::PeerId,
    },
    HistoryWriteTimerFired,
    HistoryRecordFound {
        id: p2p::ZoneHistoryRecordKind,
        value: Vec<u8>,
    },
    HistoryRecordNotFound {
        id: p2p::ZoneHistoryRecordKind,
    },
    AntiEntropyTimerFired {
        local: replica_sync::ReplicaSliceContentHashes,
    },
    RemoteSliceContentHashes {
        from: identity::PeerId,
        local: replica_sync::ReplicaSliceContentHashes,
        remote: replica_sync::ReplicaSliceContentHashes,
    },
    ReplicaSliceRequested {
        id: p2p::ReplyId,
        zone: identity::ZoneName,
        index: replica_sync::ReplicaSliceIndex,
    },
    ReplicaSliceReceived {
        from: identity::PeerId,
        zone: identity::ZoneName,
        index: replica_sync::ReplicaSliceIndex,
        data: Vec<u8>,
    },
    ReplicaSliceRequestFailed {
        from: identity::PeerId,
        index: replica_sync::ReplicaSliceIndex,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output<Op> {
    FetchSnapshotPartFromZoneMember {
        peer: identity::PeerId,
        zone: identity::ZoneName,
        part: u32,
    },
    SendSnapshotPartToZoneMember {
        id: p2p::ReplyId,
        zone: identity::ZoneName,
        part: u32,
    },
    Merge {
        bytes: Vec<u8>,
    },
    Apply {
        meta: replica_sync::OpMeta,
        op: Op,
    },
    GetHistoryRecord {
        id: p2p::ZoneHistoryRecordKind,
    },
    MergeAndWriteHistory {
        prefix: p2p::HistoryKeyPrefix,
        stored_manifest: Option<p2p::HistoryManifest>,
        stored_snapshot: Option<Vec<u8>>,
    },
    ZoneGone {
        zone: identity::ZoneName,
    },
    HistoryUnavailable {
        zone: identity::ZoneName,
    },
    PublishSliceContentHashes {
        zone: identity::ZoneName,
        hashes: replica_sync::ReplicaSliceContentHashes,
    },
    FetchReplicaSlice {
        peer: identity::PeerId,
        zone: identity::ZoneName,
        index: replica_sync::ReplicaSliceIndex,
    },
    SendReplicaSlice {
        id: p2p::ReplyId,
        zone: identity::ZoneName,
        index: replica_sync::ReplicaSliceIndex,
    },
}
