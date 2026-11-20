use serde::{Deserialize, Serialize};

use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FakeWireHello {
    pub peer: identity::Peer,
    pub game: identity::GameId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct KeyValueOp {
    pub key: u8,
    pub value: u32,
}
