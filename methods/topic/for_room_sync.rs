use crate::identity;

#[must_use]
pub fn for_room_sync(game: &identity::GameId, room: &identity::RoomName) -> identity::Topic {
    let path = format!("{game}/room/{room}/sync");
    identity::Topic::from(path)
}

#[cfg(test)]
mod tests {
    use super::for_room_sync;
    use crate::identity;

    #[test]
    fn test_usage() {
        let topic = for_room_sync(
            &identity::GameId::from("game"),
            &identity::RoomName::from("room-a"),
        );
        assert_eq!(topic.as_str(), "game/room/room-a/sync");
    }
}
