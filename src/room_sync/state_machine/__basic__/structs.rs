use std::collections::BTreeMap;

use crate::identity;
use crate::p2p;
use crate::room_sync;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LobbyRoomStatus {
    pub is_room_alive_in_lobby: bool,
    pub has_other_lobby_holders: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SliceAntiEntropyState {
    pub fetches_awaiting_reply: BTreeMap<
        identity::PeerId,
        BTreeMap<room_sync::ReplicaSliceIndex, room_sync::ElapsedSinceAppStart>,
    >,
    pub max_fetches_per_peer: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingHistoryFetch {
    pub prefix: p2p::HistoryKeyPrefix,
    pub manifest: Option<p2p::HistoryManifest>,
    pub received_chunks: BTreeMap<p2p::ChunkIndex, Vec<u8>>,
    pub records_requested_at: Option<room_sync::ElapsedSinceAppStart>,
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
