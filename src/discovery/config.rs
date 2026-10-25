use atomic_delegate_macros::atomic_delegates;
use tokio::sync::mpsc::UnboundedSender;

use crate::discovery;
use crate::identity;
use crate::p2p;

#[derive(Debug, Clone)]
pub struct Config {
    pub game: identity::GameId,
    pub display_name: identity::Username,
    pub room_size: discovery::RoomSize,
    pub deploy_policy: discovery::freenet::DeployPolicy,
    pub lease: discovery::HolderHeartbeatIntervals,
    pub history_size_limits: discovery::HistorySizeLimits,
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

#[atomic_delegates]
impl Config {
    pub async fn run(&self, endpoint: discovery::FreenetPort, channels: discovery::RunChannels) {}
}

#[rustfmt::skip]
impl Config {
    pub(crate) fn flush(
        outputs: Vec<discovery::state_machine::Output>,
        net: &UnboundedSender<p2p::NetCommand>,
        events: &UnboundedSender<discovery::Event>,
    ) {
        for output in outputs {
            match output {
                discovery::state_machine::Output::Notify(event) => {
                    let _ = events.send(event);
                }
                discovery::state_machine::Output::NetCommand(command) => {
                    let _ = net.send(command);
                }
            }
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
