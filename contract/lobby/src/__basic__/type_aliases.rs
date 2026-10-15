use std::collections::BTreeMap;

use crate::basic::newtypes::RoomName;
use crate::basic::structs::RoomRecord;

pub type LobbyState = BTreeMap<RoomName, RoomRecord>;
