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
