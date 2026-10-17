use crate::identity;
use crate::room_sync;

pub struct State<O> {
    pub room: Option<identity::RoomName>,
    pub phase: room_sync::state_machine::Phase,
    pub buffered: Vec<(room_sync::OpMeta, O)>,
    pub total: u32,
    pub parts: Vec<Option<Vec<u8>>>,
    pub members: Vec<identity::PeerId>,
    pub asked: Vec<identity::PeerId>,
    pub source: Option<identity::PeerId>,
}

impl<O> Default for State<O> {
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
        }
    }
}

// no test_usage necessary
