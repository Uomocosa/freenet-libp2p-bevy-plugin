use serde::{Deserialize, Serialize};

use crate::net_id;
use crate::p2p;

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
    Ready(net_id::Peer),
    ObservedAddr(net_id::PeerAddr),
    PeerConnected(net_id::PeerId),
    PeerDisconnected(net_id::PeerId),
    DialFailed {
        peer_id: net_id::PeerId,
        reason: String,
    },
    RelayReserved {
        relay_peer_id: net_id::PeerId,
    },
    RoomProviders {
        room: net_id::RoomName,
        peers: Vec<net_id::PeerId>,
    },
    HistoryChunk {
        id: p2p::HistoryChunkId,
        data: Vec<u8>,
    },
    Gossip {
        topic: net_id::Topic,
        from: net_id::PeerId,
        data: Vec<u8>,
    },
    Exchange {
        from: net_id::PeerId,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetCommand {
    Dial(net_id::Peer),
    ReserveRelay {
        relay_addr: net_id::PeerAddr,
    },
    SetMdns {
        mode: MdnsMode,
    },
    AddKadPeer(net_id::Peer),
    ProvideRoom {
        room: net_id::RoomName,
    },
    FindRoom {
        room: net_id::RoomName,
    },
    PutHistory {
        id: p2p::HistoryChunkId,
        data: Vec<u8>,
    },
    FetchHistory {
        id: p2p::HistoryChunkId,
    },
    FetchRoster {
        room: net_id::RoomName,
    },
    Subscribe {
        topic: net_id::Topic,
    },
    Publish {
        topic: net_id::Topic,
        data: Vec<u8>,
    },
    Exchange {
        peer_id: net_id::PeerId,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command<T> {
    Net(NetCommand),
    Send { peer_id: net_id::PeerId, payload: T },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event<T> {
    Net(NetEvent),
    Message { from: net_id::PeerId, payload: T },
    Error(String),
}
