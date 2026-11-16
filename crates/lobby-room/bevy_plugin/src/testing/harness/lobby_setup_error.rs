use thiserror::Error;

use freenet_libp2p_bevy_core::freenet_node;

use crate::freenet;

#[derive(Debug, Error)]
pub enum LobbySetupError {
    #[error("lobby setup: embedded helper node failed to start")]
    NodeStart(#[source] freenet_node::Error),
    #[error("lobby setup: contract deploy failed")]
    Deploy(#[source] freenet::Error),
    #[error("lobby setup: contract not readable after deploy")]
    NotReadableAfterDeploy(#[source] freenet::Error),
}

#[cfg(test)]
mod tests {
    use super::LobbySetupError;
    use freenet_libp2p_bevy_core::freenet_node;

    use crate::freenet;

    #[test]
    fn test_usage() {
        let start =
            LobbySetupError::NodeStart(freenet_node::Error::Bind(std::io::Error::other("no port")));
        assert!(start.to_string().contains("failed to start"));

        let deploy = LobbySetupError::Deploy(freenet::Error::ConnectionTimeout);
        assert!(deploy.to_string().contains("deploy failed"));

        let readable = LobbySetupError::NotReadableAfterDeploy(freenet::Error::ContractNotFound);
        assert!(readable.to_string().contains("not readable after deploy"));
    }
}
