use crate::identity;
use crate::p2p;
use crate::room_sync;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Idle,
    FetchingSnapshotFromRoomMember,
    FetchingHistoryFromKademlia,
    Ready,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input<Op> {
    Joined {
        room: identity::RoomName,
        prefix: p2p::HistoryKeyPrefix,
        members: Vec<identity::PeerId>,
        lobby_status: room_sync::state_machine::LobbyRoomStatus,
    },
    Members {
        members: Vec<identity::PeerId>,
        prefix: p2p::HistoryKeyPrefix,
        lobby_status: room_sync::state_machine::LobbyRoomStatus,
    },
    Remote {
        room: identity::RoomName,
        meta: room_sync::OpMeta,
        op: Op,
    },
    SnapshotPartReceivedFromRoomMember {
        room: identity::RoomName,
        part: u32,
        total: u32,
        data: Vec<u8>,
    },
    SnapshotPartRequestedByRoomMember {
        id: p2p::ReplyId,
        room: identity::RoomName,
        part: u32,
    },
    SnapshotMissingFromRoomMember,
    RequestToRoomMemberFailed {
        from: identity::PeerId,
    },
    HistoryWriteTimerFired,
    HistoryRecordFound {
        id: p2p::RoomHistoryRecordKind,
        value: Vec<u8>,
    },
    HistoryRecordNotFound {
        id: p2p::RoomHistoryRecordKind,
    },
    AntiEntropyTimerFired {
        local: room_sync::ReplicaSliceContentHashes,
    },
    RemoteSliceContentHashes {
        from: identity::PeerId,
        local: room_sync::ReplicaSliceContentHashes,
        remote: room_sync::ReplicaSliceContentHashes,
    },
    ReplicaSliceRequested {
        id: p2p::ReplyId,
        room: identity::RoomName,
        index: room_sync::ReplicaSliceIndex,
    },
    ReplicaSliceReceived {
        from: identity::PeerId,
        room: identity::RoomName,
        index: room_sync::ReplicaSliceIndex,
        data: Vec<u8>,
    },
    ReplicaSliceRequestFailed {
        from: identity::PeerId,
        index: room_sync::ReplicaSliceIndex,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output<Op> {
    FetchSnapshotPartFromRoomMember {
        peer: identity::PeerId,
        room: identity::RoomName,
        part: u32,
    },
    SendSnapshotPartToRoomMember {
        id: p2p::ReplyId,
        room: identity::RoomName,
        part: u32,
    },
    Merge {
        bytes: Vec<u8>,
    },
    Apply {
        meta: room_sync::OpMeta,
        op: Op,
    },
    GetHistoryRecord {
        id: p2p::RoomHistoryRecordKind,
    },
    MergeAndWriteHistory {
        prefix: p2p::HistoryKeyPrefix,
        stored_manifest: Option<p2p::HistoryManifest>,
        stored_snapshot: Option<Vec<u8>>,
    },
    RoomGone {
        room: identity::RoomName,
    },
    HistoryUnavailable {
        room: identity::RoomName,
    },
    PublishSliceContentHashes {
        room: identity::RoomName,
        hashes: room_sync::ReplicaSliceContentHashes,
    },
    FetchReplicaSlice {
        peer: identity::PeerId,
        room: identity::RoomName,
        index: room_sync::ReplicaSliceIndex,
    },
    SendReplicaSlice {
        id: p2p::ReplyId,
        room: identity::RoomName,
        index: room_sync::ReplicaSliceIndex,
    },
}
