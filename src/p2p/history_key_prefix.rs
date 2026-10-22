use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};

use crate::discovery;
use crate::identity;

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Deref, From,
)]
#[from(forward)]
pub struct HistoryKeyPrefix(pub String);

impl HistoryKeyPrefix {
    #[must_use]
    pub fn new(
        game: &identity::GameId,
        room: &identity::RoomName,
        created_at: discovery::UnixTime,
    ) -> Self {
        let digest = blake3::hash(
            format!(
                "{}/{}/{}",
                game.as_str(),
                room.as_str(),
                created_at.as_secs()
            )
            .as_bytes(),
        );
        Self::from(digest.to_hex().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::HistoryKeyPrefix;
    use crate::discovery;
    use crate::identity;

    #[test]
    fn test_usage() {
        let prefix = HistoryKeyPrefix::new(
            &identity::GameId::from("game"),
            &identity::RoomName::from("room"),
            discovery::UnixTime::from(7),
        );
        assert_eq!(prefix.len(), 64);
    }
}
