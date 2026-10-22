use atomic_delegate_macros::atomic_delegates;

use crate::discovery;
use crate::identity;

#[derive(Debug, Clone)]
pub struct Config {
    pub game: identity::GameId,
    pub display_name: identity::Username,
    pub room_size: discovery::RoomSize,
    pub deploy_policy: discovery::freenet::DeployPolicy,
    pub lease: discovery::HolderHeartbeatIntervals,
    pub history_size_limits: discovery::HistorySizeLimits,
}

#[atomic_delegates]
impl Config {
    pub async fn run(&self, endpoint: discovery::FreenetPort, channels: discovery::RunChannels) {}
}

impl Default for Config {
    fn default() -> Self {
        Self {
            game: identity::GameId::from("test"),
            display_name: identity::Username::from("anonymous"),
            room_size: discovery::RoomSize::DEFAULT,
            deploy_policy: discovery::freenet::DeployPolicy::default(),
            lease: discovery::HolderHeartbeatIntervals::default(),
            history_size_limits: discovery::HistorySizeLimits::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    use crate::discovery;

    #[test]
    fn test_usage() {
        let config = Config::default();
        let params = config.game.contract_params();
        assert_eq!(params, b"test".to_vec());
        assert_eq!(config.room_size, discovery::RoomSize::DEFAULT);
        assert_eq!(
            config.deploy_policy,
            discovery::freenet::DeployPolicy::FetchOrDeploy
        );
    }
}
