use serde::{Deserialize, Serialize};

use crate::identity;
use crate::p2p;
use crate::room_sync;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode {
    Tcp,
    Quic,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MdnsMode {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetEvent {
    Ready(identity::Peer),
    ObservedAddr(identity::PeerAddr),
    PeerConnected(identity::PeerId),
    PeerDisconnected(identity::PeerId),
    DialFailed {
        peer_id: identity::PeerId,
        reason: String,
    },
    RelayReserved {
        relay_peer_id: identity::PeerId,
    },
    HistoryChunk {
        id: p2p::HistoryChunkId,
        data: Vec<u8>,
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
    SnapshotFailed {
        from: identity::PeerId,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetCommand {
    Dial(identity::Peer),
    ReserveRelay {
        relay_addr: identity::PeerAddr,
    },
    SetMdns {
        mode: MdnsMode,
    },
    AddKadPeer(identity::Peer),
    PutHistory {
        id: p2p::HistoryChunkId,
        data: Vec<u8>,
    },
    FetchHistory {
        id: p2p::HistoryChunkId,
    },
    Subscribe {
        topic: identity::Topic,
    },
    Unsubscribe {
        topic: identity::Topic,
    },
    SubscribeRoom {
        room: identity::RoomName,
    },
    UnsubscribeRoom {
        room: identity::RoomName,
    },
    FetchSnapshot {
        peer_id: identity::PeerId,
        room: identity::RoomName,
        part: u32,
    },
    FetchReplicaSlice {
        peer_id: identity::PeerId,
        room: identity::RoomName,
        index: room_sync::ReplicaSliceIndex,
    },
    RespondSnapshot {
        id: p2p::ReplyId,
        frame: p2p::SnapshotResponse,
    },
    Publish {
        topic: identity::Topic,
        data: Vec<u8>,
    },
    PeerExchange {
        peer_id: identity::PeerId,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command<Payload> {
    Net(NetCommand),
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
    Net(NetEvent),
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
