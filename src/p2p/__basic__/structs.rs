use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver as WatchReceiver;

use crate::identity;
use crate::p2p;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryChunkId {
    pub prefix: p2p::HistoryKeyPrefix,
    pub content_hash: p2p::ContentHash,
    pub chunk: p2p::ChunkIndex,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryManifest {
    pub lamport_max: u64,
    pub chunk_count: u32,
    pub content_hash: p2p::ContentHash,
}

pub struct SwarmChannels {
    pub commands: UnboundedSender<p2p::Libp2pCommand>,
    pub events: Receiver<p2p::Libp2pEvent>,
    pub ready: WatchReceiver<Option<identity::Peer>>,
    pub observed: WatchReceiver<Option<Vec<identity::PeerAddr>>>,
}
