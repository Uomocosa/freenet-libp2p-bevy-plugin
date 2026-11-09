use bevy::prelude::Component;
use derive_more::{Deref, From};

use crate::identity;

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref, From)]
pub struct Room(pub identity::RoomName);

#[rustfmt::skip]
impl PartialEq<identity::RoomName> for Room {
    fn eq(&self, other: &identity::RoomName) -> bool {
        **self == *other
    }
}

#[cfg(test)]
mod tests {
    use super::Room;
    use crate::identity;

    #[test]
    fn test_usage() {
        let room = Room::from(identity::RoomName::from("lobby"));
        assert_eq!(room, identity::RoomName::from("lobby"));
        assert_ne!(room, identity::RoomName::from("other"));
    }
}
