use serde::{Deserialize, Serialize};

use crate::direct_dial;
use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnownPeer {
    pub peer: identity::Peer,
    pub kind: direct_dial::KnownPeerKind,
}
