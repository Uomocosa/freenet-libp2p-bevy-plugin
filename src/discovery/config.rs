use crate::discovery;
use crate::identity;

#[derive(Debug, Clone)]
pub struct Config {
    pub game: identity::GameId,
    pub display_name: identity::Username,
    pub room_size: discovery::RoomSize,
    pub deploy_policy: discovery::freenet::DeployPolicy,
    pub auto_subscribe: discovery::AutoSubscribe,
    pub lease: discovery::LeaseTimings,
    pub history_size_limits: discovery::HistorySizeLimits,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            game: identity::GameId::from("test"),
            display_name: identity::Username::from("anonymous"),
            room_size: discovery::RoomSize::DEFAULT,
            deploy_policy: discovery::freenet::DeployPolicy::default(),
            auto_subscribe: discovery::AutoSubscribe::default(),
            lease: discovery::LeaseTimings::default(),
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
        let params = discovery::freenet::contract_params(&config.game);
        assert_eq!(params, b"test".to_vec());
        assert_eq!(config.room_size, discovery::RoomSize::DEFAULT);
        assert_eq!(
            config.deploy_policy,
            discovery::freenet::DeployPolicy::FetchOrDeploy
        );
        assert_eq!(config.auto_subscribe, discovery::AutoSubscribe::Enabled);
    }
}
