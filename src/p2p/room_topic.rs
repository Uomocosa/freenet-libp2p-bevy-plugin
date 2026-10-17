use crate::identity;

#[must_use]
pub fn room_topic(game: &identity::GameId, room: &identity::RoomName) -> identity::Topic {
    identity::Topic::from(format!("{game}/room/{room}"))
}

#[cfg(test)]
mod tests {
    use super::room_topic;
    use crate::identity;

    #[test]
    fn test_usage() {
        let topic = room_topic(
            &identity::GameId::from("game/id"),
            &identity::RoomName::from("room-a"),
        );
        assert_eq!(topic.as_str(), "game/id/room/room-a");
    }
}
