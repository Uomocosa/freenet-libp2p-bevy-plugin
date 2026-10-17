use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::UnixTime;
use crate::basic::newtypes::PeerId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heartbeat {
    pub addrs: Vec<String>,
    pub updated_at: UnixTime,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Room {
    pub room_size: u16,
    pub created_at: UnixTime,
    pub members: BTreeMap<PeerId, Heartbeat>,
}
