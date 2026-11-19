use crate::testing;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LobbySetup {
    Ensured,
    Absent,
}

impl LobbySetup {
    #[must_use]
    pub fn lobby_token(self, stamp: u64) -> testing::harness::LobbyToken {
        match self {
            Self::Ensured => testing::harness::LobbyToken::from("lobby-e2e/ensured"),
            Self::Absent => testing::harness::LobbyToken::from(format!("lobby-e2e/absent-{stamp}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LobbySetup;

    #[test]
    fn test_usage() {
        let ensured = LobbySetup::Ensured;
        assert_eq!(
            ensured.lobby_token(1).to_string(),
            ensured.lobby_token(2).to_string()
        );
        assert_eq!(ensured.lobby_token(1).to_string(), "lobby-e2e/ensured");

        let absent = LobbySetup::Absent;
        assert_ne!(
            absent.lobby_token(1).to_string(),
            absent.lobby_token(2).to_string()
        );
        assert_eq!(absent.lobby_token(7).to_string(), "lobby-e2e/absent-7");
    }
}
