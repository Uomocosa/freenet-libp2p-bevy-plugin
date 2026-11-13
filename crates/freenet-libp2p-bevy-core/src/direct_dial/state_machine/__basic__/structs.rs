use serde::{Deserialize, Serialize};

use crate::direct_dial;
use crate::identity;
use crate::time;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DialKnownPeer {
    pub peer: identity::Peer,
    pub kind: direct_dial::KnownPeerKind,
    pub failed_attempts: u32,
    pub next_attempt: time::Deadline,
}
