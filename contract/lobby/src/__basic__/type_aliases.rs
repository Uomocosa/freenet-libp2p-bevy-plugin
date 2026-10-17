use std::collections::BTreeMap;

use crate::basic::newtypes::RoomName;
use crate::basic::structs::Room;

pub type Lobby = BTreeMap<RoomName, Room>;
