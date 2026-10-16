use std::collections::BTreeMap;

use crate::discovery;
use crate::identity;

pub type Lobby = BTreeMap<identity::RoomName, discovery::Room>;
pub type Members = BTreeMap<identity::PeerId, discovery::Member>;
