use thiserror::Error;

/// Why the lobby setup could not make the run's contract readable.
#[derive(Debug, Error)]
pub enum LobbySetupError {
    #[error("lobby setup: embedded helper node failed to start: {0}")]
    NodeStart(String),
    #[error("lobby setup: contract deploy failed")]
    Deploy(#[source] freenet_libp2p_bevy_plugin::lobby_room::freenet::Error),
    #[error("lobby setup: contract not readable after deploy")]
    NotReadableAfterDeploy(#[source] freenet_libp2p_bevy_plugin::lobby_room::freenet::Error),
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_plugin::lobby_room::freenet;

    use super::LobbySetupError;

    #[test]
    fn test_usage() {
        let start = LobbySetupError::NodeStart("no port".to_string());
        assert!(start.to_string().contains("failed to start"));

        let deploy = LobbySetupError::Deploy(freenet::Error::ConnectionTimeout);
        assert!(deploy.to_string().contains("deploy failed"));

        let readable = LobbySetupError::NotReadableAfterDeploy(freenet::Error::ContractNotFound);
        assert!(readable.to_string().contains("not readable after deploy"));
    }
}
