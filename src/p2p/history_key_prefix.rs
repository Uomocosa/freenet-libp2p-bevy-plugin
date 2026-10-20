use crate::discovery;
use crate::identity;
use crate::p2p;

#[must_use]
pub fn history_key_prefix(
    game: &identity::GameId,
    room: &identity::RoomName,
    created_at: discovery::UnixTime,
) -> p2p::HistoryKeyPrefix {
    let digest = blake3::hash(
        format!(
            "{}/{}/{}",
            game.as_str(),
            room.as_str(),
            created_at.as_secs()
        )
        .as_bytes(),
    );
    p2p::HistoryKeyPrefix::from(digest.to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::history_key_prefix;
    use crate::discovery;
    use crate::identity;

    #[test]
    fn test_usage() {
        let prefix = history_key_prefix(
            &identity::GameId::from("game"),
            &identity::RoomName::from("room-a"),
            discovery::UnixTime::from_secs(7),
        );
        assert_eq!(prefix.len(), 64);
    }

    #[test]
    fn test_usage_generation_scoped() {
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room-a");
        let first = history_key_prefix(&game, &room, discovery::UnixTime::from_secs(1));
        let second = history_key_prefix(&game, &room, discovery::UnixTime::from_secs(2));
        assert_ne!(first, second);
    }
}
