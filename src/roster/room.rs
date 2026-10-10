use bevy::prelude::Resource;
use derive_more::Deref;

use crate::net_id;
use crate::roster::basic::constants;

#[derive(Resource, Debug, Clone, PartialEq, Eq, Deref)]
pub struct Room(pub net_id::RoomName);

impl Default for Room {
    fn default() -> Self {
        Self(net_id::RoomName(constants::DEFAULT_ROOM.to_string()))
    }
}

impl Room {
    #[must_use]
    pub const fn new(name: net_id::RoomName) -> Self {
        Self(name)
    }
}

#[cfg(test)]
mod tests {
    use super::Room;
    use crate::net_id;

    #[test]
    fn test_usage() {
        assert_eq!(Room::default().as_str(), "default");
        assert_eq!(Room::new(net_id::RoomName::from("alpha")).as_str(), "alpha");
    }
}
