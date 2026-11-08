use bevy::prelude::App;

use crate::discovery;
use crate::p2p;
use crate::testing;

pub fn build_plugin<Payload: p2p::Message>(
    plugin: &testing::FakeNetworkPlugin<Payload>,
    app: &mut App,
) {
    let network = plugin.network;
    match network {
        testing::Network::Mainnet => {}
        testing::Network::LocalGateways => {
            tracing::error!(target: "testing", "local-gateways network is not implemented yet");
        }
        testing::Network::DirectDial => {
            app.insert_resource(discovery::LobbyTaskOverride {
                orchestrator_port: None,
                peer: plugin.peer.clone(),
            });
        }
        testing::Network::Fake => {
            let Some(port) = plugin.orchestrator_port else {
                tracing::error!(
                    target: "testing",
                    "fake network needs --orchestrator-port"
                );
                return;
            };
            app.insert_resource(testing::FakeTaskHandles::default());
            app.insert_resource(p2p::SwarmTaskOverride::<Payload> {
                orchestrator_port: port,
                peer: plugin.peer.clone(),
                marker: std::marker::PhantomData,
            });
            app.insert_resource(discovery::LobbyTaskOverride {
                orchestrator_port: Some(port),
                peer: plugin.peer.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::build_plugin;
    use crate::discovery;
    use crate::p2p;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let args = testing::NetworkArgs {
            network: testing::Network::Mainnet,
            orchestrator_port: None,
        };
        let plugin = testing::FakeNetworkPlugin::<u8>::from(&args);
        build_plugin(&plugin, &mut app);
        assert!(
            app.world()
                .get_resource::<p2p::SwarmTaskOverride<u8>>()
                .is_none()
        );
    }

    #[test]
    fn direct_dial_keeps_the_real_swarm_with_an_idle_lobby() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let args = testing::NetworkArgs {
            network: testing::Network::DirectDial,
            orchestrator_port: None,
        };
        let plugin = testing::FakeNetworkPlugin::<u8>::from(&args);
        build_plugin(&plugin, &mut app);
        assert!(
            app.world()
                .get_resource::<p2p::SwarmTaskOverride<u8>>()
                .is_none()
        );
        let lobby = app
            .world()
            .get_resource::<discovery::LobbyTaskOverride>()
            .expect("idle lobby override");
        assert_eq!(lobby.orchestrator_port, None);
    }
}
