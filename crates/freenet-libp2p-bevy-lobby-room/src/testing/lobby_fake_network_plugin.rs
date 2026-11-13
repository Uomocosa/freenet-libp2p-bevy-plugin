use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::{App, Plugin};

use crate::identity;
use crate::testing;

pub struct LobbyFakeNetworkPlugin {
    pub(crate) network: testing::Network,
    pub(crate) orchestrator_port: Option<u16>,
    pub(crate) peer: identity::Peer,
}

impl LobbyFakeNetworkPlugin {
    #[must_use]
    pub fn new<Payload>(plugin: &testing::FakeNetworkPlugin<Payload>) -> Self {
        Self {
            network: plugin.network,
            orchestrator_port: plugin.orchestrator_port,
            peer: plugin.peer.clone(),
        }
    }
}

#[rustfmt::skip]
impl Plugin for LobbyFakeNetworkPlugin {
    fn build(&self, app: &mut App) { self.build_plugin(app); }
}

#[atomic_delegates]
impl LobbyFakeNetworkPlugin {
    pub fn build_plugin(&self, app: &mut App) {}
}

#[cfg(test)]
mod tests {
    use crate::testing;

    #[test]
    fn test_usage() {
        let args = testing::NetworkArgs {
            network: testing::Network::Mainnet,
            orchestrator_port: None,
        };
        let plugin = testing::FakeNetworkPlugin::<u8>::from(&args);
        let _ = super::LobbyFakeNetworkPlugin::new(&plugin);
    }
}
