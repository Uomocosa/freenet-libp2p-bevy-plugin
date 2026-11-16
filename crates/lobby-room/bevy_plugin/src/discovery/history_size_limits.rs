use crate::p2p;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistorySizeLimits {
    pub per_process_bytes: p2p::HistoryByteCount,
    pub per_room_bytes: p2p::HistoryByteCount,
    pub max_held_rooms: usize,
}

impl Default for HistorySizeLimits {
    fn default() -> Self {
        Self {
            per_process_bytes: p2p::HistoryByteCount::from(256u64 * 1024 * 1024),
            per_room_bytes: p2p::HistoryByteCount::from(64u64 * 1024 * 1024),
            max_held_rooms: 8,
        }
    }
}

// no test_usage necessary
