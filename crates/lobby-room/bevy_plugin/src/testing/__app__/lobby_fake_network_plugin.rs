use bevy::prelude::{App, Plugin};

use crate::discovery;
use crate::identity;
use crate::testing;
use bevy::prelude::{IntoScheduleConfigs, Update};
use lele_bevy_state_machine::StateMachineSystems;

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

impl Plugin for LobbyFakeNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            testing::bevy_systems::decode_extension_effects.before(
                StateMachineSystems::collect_inputs::<discovery::state_machine::DiscoveryState>(),
            ),
        );
        match self.network {
            testing::Network::Mainnet | testing::Network::LocalGateways => {}
            testing::Network::DirectDial => {
                app.insert_resource(discovery::LobbyTaskOverride {
                    orchestrator_port: None,
                    peer: self.peer.clone(),
                });
            }
            testing::Network::Fake => {
                let Some(port) = self.orchestrator_port else {
                    return;
                };
                app.insert_resource(discovery::LobbyTaskOverride {
                    orchestrator_port: Some(port),
                    peer: self.peer.clone(),
                });
            }
        }
    }
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

#[cfg(test)]
mod build_plugin_tests {
    use crate::testing;

    #[test]
    fn test_usage() {
        let args = testing::NetworkArgs {
            network: testing::Network::Mainnet,
            orchestrator_port: None,
        };
        let plugin = testing::FakeNetworkPlugin::<u8>::from(&args);
        let mut app = bevy::prelude::App::new();
        super::Plugin::build(&testing::LobbyFakeNetworkPlugin::new(&plugin), &mut app);
    }
}
