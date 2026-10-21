use std::collections::{BTreeMap, BTreeSet};

use crate::identity;
use crate::p2p;
use crate::room_sync;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LobbyRoomStatus {
    pub is_room_alive_in_lobby: bool,
    pub has_other_lobby_holders: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MemberSnapshotFetchState {
    pub total_parts: u32,
    pub received_parts: BTreeMap<u32, Vec<u8>>,
    pub members: Vec<identity::PeerId>,
    pub asked_members: Vec<identity::PeerId>,
    pub current_source: Option<identity::PeerId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SliceAntiEntropyState {
    pub fetches_awaiting_reply: BTreeMap<identity::PeerId, BTreeSet<room_sync::ReplicaSliceIndex>>,
    pub max_fetches_per_peer: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingHistoryFetch {
    pub prefix: p2p::HistoryKeyPrefix,
    pub manifest: Option<p2p::HistoryManifest>,
    pub received_chunks: BTreeMap<p2p::ChunkIndex, Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KademliaHistoryState {
    pub is_enabled: bool,
    pub prefix: Option<p2p::HistoryKeyPrefix>,
    pub lobby_status: LobbyRoomStatus,
    pub fetch_retries: u32,
    pub max_fetch_retries: u32,
    pub per_room_bytes: p2p::HistoryByteCount,
    pub record_ttl: std::time::Duration,
    pub pending_fetch: Option<PendingHistoryFetch>,
}
