use crate::identity;
use crate::p2p;
use crate::room_sync;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Idle,
    Fetching,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input<Op> {
    Joined {
        room: identity::RoomName,
        members: Vec<identity::PeerId>,
    },
    Members {
        members: Vec<identity::PeerId>,
    },
    Remote {
        room: identity::RoomName,
        meta: room_sync::OpMeta,
        op: Op,
    },
    Snapshot {
        room: identity::RoomName,
        part: u32,
        total: u32,
        data: Vec<u8>,
    },
    Fetch {
        id: p2p::ReplyId,
        room: identity::RoomName,
        part: u32,
    },
    Tick,
    Failed {
        from: identity::PeerId,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output<Op> {
    Fetch {
        peer: identity::PeerId,
        room: identity::RoomName,
        part: u32,
    },
    Snapshot {
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
