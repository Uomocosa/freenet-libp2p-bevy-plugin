use std::collections::BTreeMap;

use crate::identity;
use crate::room_sync;

pub struct State<Op> {
    pub room: Option<identity::RoomName>,
    pub phase: room_sync::state_machine::Phase,
    pub buffered_ops: Vec<(room_sync::OpMeta, Op)>,
    pub member_snapshot: room_sync::state_machine::MemberSnapshotFetchState,
    pub slice_anti_entropy: room_sync::state_machine::SliceAntiEntropyState,
    pub kademlia_history: room_sync::state_machine::KademliaHistoryState,
}

impl<Op> Default for State<Op> {
    fn default() -> Self {
        let history_defaults = room_sync::HistoryConfig::default();
        Self {
            room: None,
            phase: room_sync::state_machine::Phase::Idle,
            buffered_ops: Vec::new(),
            member_snapshot: room_sync::state_machine::MemberSnapshotFetchState::default(),
            slice_anti_entropy: room_sync::state_machine::SliceAntiEntropyState {
                fetches_awaiting_reply: BTreeMap::new(),
                max_fetches_per_peer: room_sync::AntiEntropyConfig::default()
                    .max_slice_fetches_per_peer,
            },
            kademlia_history: room_sync::state_machine::KademliaHistoryState {
                is_enabled: false,
                prefix: None,
                lobby_status: room_sync::state_machine::LobbyRoomStatus::default(),
                fetch_retries: 0,
                max_fetch_retries: history_defaults.max_fetch_retries,
                per_room_bytes: history_defaults.per_room_bytes,
                record_ttl: history_defaults.record_ttl,
                pending_fetch: None,
            },
        }
    }
}

// no test_usage necessary
