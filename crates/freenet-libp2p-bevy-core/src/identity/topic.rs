use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Reflect;
use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};

use crate::identity;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Deref,
    Display,
    From,
    Reflect,
)]
#[reflect(Hash)]
#[from(forward)]
pub struct Topic(pub String);

#[atomic_delegates]
impl Topic {
    pub fn from_room(game: &identity::GameId, room: &identity::RoomName) -> Topic {}
    pub fn from_room_sync(game: &identity::GameId, room: &identity::RoomName) -> Topic {}
}

#[cfg(test)]
mod tests {
    use super::Topic;

    #[test]
    fn test_usage() {
        assert_eq!(Topic::from("game/room").as_str(), "game/room");
    }
}
