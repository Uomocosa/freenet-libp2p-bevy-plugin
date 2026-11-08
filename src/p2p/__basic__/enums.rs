use std::collections::BTreeMap;

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

use crate::identity;
use crate::p2p;
use crate::room_sync;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Libp2pEvent {
    Ready(identity::Peer),
    ObservedAddr(identity::PeerAddr),
    PeerConnected(identity::PeerId),
    PeerDisconnected(identity::PeerId),
    PeerListenAddrs {
        peer_id: identity::PeerId,
        addrs: Vec<identity::PeerAddr>,
    },
    Libp2pConnectionEstablished {
        peer_id: identity::PeerId,
        kind: p2p::ConnectionKind,
    },
    DialFailed {
        peer_id: identity::PeerId,
        reason: String,
    },
    RelayReserved {
        relay_peer_id: identity::PeerId,
    },
    HistoryRecordStored {
        id: p2p::RoomHistoryRecordKind,
    },
    HistoryRecordPutFailed {
        id: p2p::RoomHistoryRecordKind,
        reason: String,
    },
    HistoryRecordFound {
        id: p2p::RoomHistoryRecordKind,
        value: Vec<u8>,
    },
    HistoryRecordNotFound {
        id: p2p::RoomHistoryRecordKind,
    },
    HistoryStoreMeasured {
        bytes_by_prefix: BTreeMap<p2p::HistoryKeyPrefix, p2p::HistoryByteCount>,
    },
    Gossip {
        topic: identity::Topic,
        from: identity::PeerId,
        data: Vec<u8>,
    },
    PeerExchange {
        from: identity::PeerId,
        data: Vec<u8>,
    },
    SnapshotRequest {
        id: p2p::ReplyId,
        from: identity::PeerId,
        room: identity::RoomName,
        part: u32,
    },
    SnapshotPart {
        from: identity::PeerId,
        room: identity::RoomName,
        part: u32,
        total: u32,
        data: Vec<u8>,
    },
    SnapshotMissing {
        from: identity::PeerId,
        room: identity::RoomName,
    },
    RoomSyncRequestFailed {
        from: identity::PeerId,
        request: p2p::SnapshotRequest,
    },
    ReplicaSliceRequest {
        id: p2p::ReplyId,
        from: identity::PeerId,
        room: identity::RoomName,
        index: room_sync::ReplicaSliceIndex,
    },
    ReplicaSlice {
        from: identity::PeerId,
        room: identity::RoomName,
        index: room_sync::ReplicaSliceIndex,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkError {
    WrongChunkCount { expected: u32, actual: usize },
    HashMismatch,
    Codec(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotRequest {
    Fetch {
        room: identity::RoomName,
        part: u32,
    },
    FetchReplicaSlice {
        room: identity::RoomName,
        index: room_sync::ReplicaSliceIndex,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotResponse {
    Part {
        room: identity::RoomName,
        part: u32,
        total: u32,
        data: Vec<u8>,
    },
    Missing {
        room: identity::RoomName,
    },
    ReplicaSlice {
        room: identity::RoomName,
        index: room_sync::ReplicaSliceIndex,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command<Payload> {
    Net(p2p::Libp2pCommand),
    Send {
        peer_id: identity::PeerId,
        payload: Payload,
    },
    PublishInJoinedRoom {
        payload: Payload,
    },
    PublishToSpecificRoom {
        room: identity::RoomName,
        payload: Payload,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event<Payload> {
    Libp2p(Libp2pEvent),
    Message {
        from: identity::PeerId,
        payload: Payload,
    },
    RoomMessage {
        room: identity::RoomName,
        from: identity::PeerId,
        payload: Payload,
    },
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum PeerConnectionStatus {
    Connected,
    NotConnected,
}
