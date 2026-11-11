use super::LobbyToken;

/// What an e2e run expects of the lobby contract behind its game token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LobbySetup {
    /// One shared, already-deployed contract (`lobby-e2e/ensured`): the harness
    /// makes sure it exists before any peer spawns, so normal runs never race
    /// the empty-seed boot deploy.
    Ensured,
    /// A never-seen key (`lobby-e2e/absent-<stamp>`): the harness does not
    /// deploy, and the peers race the boot deploy; that race and the
    /// subscription heal are the app behaviour under test in the dedicated
    /// Absent scenarios.
    Absent,
}

impl LobbySetup {
    /// The lobby token this setup pins for one run at `stamp`.
    #[must_use]
    pub fn lobby_token(self, stamp: u64) -> LobbyToken {
        match self {
            Self::Ensured => LobbyToken::from("lobby-e2e/ensured"),
            Self::Absent => LobbyToken::from(format!("lobby-e2e/absent-{stamp}")),
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
