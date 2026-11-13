use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Resource;

use crate::discovery;
use crate::freenet;
use crate::identity;
use crate::p2p;

#[derive(Resource, Debug, Clone)]
pub struct Config {
    pub game: identity::GameId,
    pub display_name: identity::Username,
    pub room_size: discovery::RoomSize,
    pub deploy_policy: freenet::DeployPolicy,
    pub lease: discovery::HolderHeartbeatIntervals,
    pub history_size_limits: discovery::HistorySizeLimits,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            game: identity::GameId::from("test"),
            display_name: identity::Username::from("anonymous"),
            room_size: discovery::RoomSize::DEFAULT,
            deploy_policy: freenet::DeployPolicy::default(),
            lease: discovery::HolderHeartbeatIntervals::default(),
            history_size_limits: discovery::HistorySizeLimits::default(),
        }
    }
}

#[atomic_delegates]
impl Config {
    pub async fn run(
        &self,
        endpoint: discovery::FreenetPort,
        swarm: p2p::SwarmChannels,
        target: tokio::sync::watch::Receiver<Vec<discovery::state_machine::HeartbeatTarget>>,
        lobbies: tokio::sync::mpsc::UnboundedSender<discovery::Lobby>,
        peers: tokio::sync::mpsc::UnboundedSender<Vec<freenet::FreenetPeer>>,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    use crate::discovery;
    use crate::freenet;

    #[test]
    fn test_usage() {
        let config = Config::default();
        let params = config.game.contract_params();
        assert_eq!(params, b"test".to_vec());
        assert_eq!(config.room_size, discovery::RoomSize::DEFAULT);
        assert_eq!(config.deploy_policy, freenet::DeployPolicy::FetchOrDeploy);
    }
}
