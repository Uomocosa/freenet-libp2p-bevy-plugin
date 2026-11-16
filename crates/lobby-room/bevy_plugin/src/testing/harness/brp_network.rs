use std::path::PathBuf;

use atomic_delegate_macros::atomic_delegates;
use freenet_libp2p_bevy_core::freenet_node;
use freenet_libp2p_bevy_core::testing::harness as core_harness;

use crate::testing;

pub struct BrpNetwork {
    pub(crate) network: testing::Network,
    pub(crate) token: String,
    pub deadlines: core_harness::Deadlines,
    pub(crate) run_dir: PathBuf,
    pub(crate) display: Option<core_harness::E2EDisplay>,
    pub(crate) orchestrator: Option<testing::FakeNetworkServer>,
    pub(crate) _helper_nodes: Vec<freenet_node::NodeDir>,
}

impl BrpNetwork {
    pub async fn prepare() -> Result<Self, String> {
        Self::prepare_for(testing::Network::from_env()).await
    }
    pub async fn prepare_for(network: testing::Network) -> Result<Self, String> {
        Self::try_from_network_setup(network, testing::harness::LobbySetup::Ensured).await
    }
}

#[atomic_delegates]
impl BrpNetwork {
    pub async fn try_from_network_setup(
        network: testing::Network,
        setup: testing::harness::LobbySetup,
    ) -> Result<Self, String> {
    }
}

#[rustfmt::skip]
impl BrpNetwork {
    #[must_use]
    pub fn peer_args(&self, username: &str, window: (i32, i32)) -> Vec<String> { self.peer_args_with_token(username, window, &self.token) }
    #[must_use]
    pub fn display_name(&self) -> Option<&str> { self.display.as_ref().map(|display| display.name.as_str()) }
    #[must_use]
    pub fn recording(&self, test_name: &str) -> core_harness::RunRecording { core_harness::RunRecording::from_env(&self.run_dir, test_name, self.display_name()) }
}

impl BrpNetwork {
    #[must_use]
    pub const fn network_arg(&self) -> &'static str {
        match self.network {
            testing::Network::Fake => "fake",
            testing::Network::Mainnet => "mainnet",
            testing::Network::LocalGateways => "local-gateways",
            testing::Network::DirectDial => "direct-dial",
        }
    }
}

#[atomic_delegates]
impl BrpNetwork {
    #[must_use]
    pub fn peer_args_with_token(
        &self,
        username: &str,
        window: (i32, i32),
        token: &str,
    ) -> Vec<String> {
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::BrpNetwork;
    use crate::testing;
    use freenet_libp2p_bevy_core::testing::harness as core_harness;

    fn test_network() -> BrpNetwork {
        BrpNetwork {
            network: testing::Network::Fake,
            token: "brp-1".to_string(),
            deadlines: core_harness::Deadlines(1.0),
            run_dir: PathBuf::from("/tmp/brp-test"),
            display: None,
            orchestrator: None,
            _helper_nodes: Vec::new(),
        }
    }

    #[test]
    fn test_usage() {
        let net = test_network();
        assert_eq!(net.network_arg(), "fake");
        let args = net.peer_args("ada", (10, 20));
        assert!(args.contains(&"--token".to_string()));
        assert!(args.contains(&"brp-1".to_string()));
        assert!(args.contains(&"10,20".to_string()));
        assert_eq!(net.display_name(), None);
        assert!(
            net.recording("probe")
                .clip_path()
                .ends_with("e2e-probe.mp4")
        );
    }
}
