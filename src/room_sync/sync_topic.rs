use crate::identity;

#[must_use]
pub fn sync_topic(game: &identity::GameId, room: &identity::RoomName) -> identity::Topic {
    identity::Topic::from(format!("{game}/room/{room}/sync"))
}

#[cfg(test)]
mod tests {
    use super::sync_topic;
    use crate::identity;

    #[test]
    fn test_usage() {
        let topic = sync_topic(
            &identity::GameId::from("game"),
            &identity::RoomName::from("room-a"),
        );
        assert_eq!(topic.as_str(), "game/room/room-a/sync");
    }
}
