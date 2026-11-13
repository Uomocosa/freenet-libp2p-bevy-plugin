use serde::{Deserialize, Serialize};

use crate::identity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnownPeerKind {
    Friend,
    PreviouslyConnected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FriendPresence {
    Offline,
    Online,
    InRoom(identity::RoomName),
}
