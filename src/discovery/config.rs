use crate::discovery;
use crate::identity;

#[derive(Debug, Clone)]
pub struct Config {
    pub game: identity::GameId,
    pub display_name: identity::Username,
    pub timing: discovery::Timing,
    pub room_size: u16,
    pub deploy_policy: discovery::freenet::DeployPolicy,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            game: identity::GameId::from("test"),
            display_name: identity::Username::from("anonymous"),
            timing: discovery::Timing::default(),
            room_size: 8,
            deploy_policy: discovery::freenet::DeployPolicy::default(),
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
        assert_eq!(config.room_size, 8);
        assert_eq!(
            config.deploy_policy,
            discovery::freenet::DeployPolicy::FetchOrDeploy
        );
    }
}
