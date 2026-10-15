use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::UnixTime;
use crate::basic::newtypes::RemotePeerId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Presence {
    pub addrs: Vec<String>,
    pub updated_at: UnixTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomRecord {
    pub capacity: u16,
    pub members: BTreeMap<RemotePeerId, Presence>,
}
