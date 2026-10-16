use bevy::prelude::Resource;
use derive_more::Deref;

use crate::identity;
use crate::roster::basic::constants;

#[derive(Resource, Debug, Clone, PartialEq, Eq, Deref)]
pub struct ActiveRoom(pub identity::RoomName);

impl Default for ActiveRoom {
    fn default() -> Self {
        Self(identity::RoomName::from(constants::DEFAULT_ROOM))
    }
}

impl ActiveRoom {
    #[must_use]
    pub const fn new(name: identity::RoomName) -> Self {
        Self(name)
    }
}

#[cfg(test)]
mod tests {
    use super::ActiveRoom;
    use crate::identity;

    #[test]
    fn test_usage() {
        assert_eq!(ActiveRoom::default().as_str(), "default");
        assert_eq!(
            ActiveRoom::new(identity::RoomName::from("alpha")).as_str(),
            "alpha"
        );
    }
}
