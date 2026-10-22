use crate::identity;

#[must_use]
pub fn for_room(game: &identity::GameId, room: &identity::RoomName) -> identity::Topic {
    let path = format!("{game}/room/{room}");
    identity::Topic::from(path)
}

#[cfg(test)]
mod tests {
    use super::for_room;
    use crate::identity;

    #[test]
    fn test_usage() {
        let topic = for_room(
            &identity::GameId::from("game/id"),
            &identity::RoomName::from("room-a"),
        );
        assert_eq!(topic.as_str(), "game/id/room/room-a");
    }
}
