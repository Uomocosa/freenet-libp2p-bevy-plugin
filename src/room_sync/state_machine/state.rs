use std::collections::{BTreeMap, BTreeSet};

use crate::identity;
use crate::room_sync;

pub struct State<Op> {
    pub room: Option<identity::RoomName>,
    pub phase: room_sync::state_machine::Phase,
    pub buffered: Vec<(room_sync::OpMeta, Op)>,
    pub total: u32,
    pub parts: Vec<Option<Vec<u8>>>,
    pub members: Vec<identity::PeerId>,
    pub asked: Vec<identity::PeerId>,
    pub source: Option<identity::PeerId>,
    pub slice_fetches_awaiting_reply:
        BTreeMap<identity::PeerId, BTreeSet<room_sync::ReplicaSliceIndex>>,
    pub max_slice_fetches_per_peer: usize,
}

impl<Op> Default for State<Op> {
    fn default() -> Self {
        Self {
            room: None,
            phase: room_sync::state_machine::Phase::Idle,
            buffered: Vec::new(),
            total: 0,
            parts: Vec::new(),
            members: Vec::new(),
            asked: Vec::new(),
            source: None,
            slice_fetches_awaiting_reply: BTreeMap::new(),
            max_slice_fetches_per_peer: room_sync::AntiEntropyConfig::default()
                .max_slice_fetches_per_peer,
        }
    }
}

// no test_usage necessary
