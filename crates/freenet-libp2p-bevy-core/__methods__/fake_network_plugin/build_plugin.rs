use bevy::prelude::App;

use crate::p2p;
use crate::testing;

pub fn build_plugin<Payload: p2p::traits::Message>(
    plugin: &testing::FakeNetworkPlugin<Payload>,
    app: &mut App,
) {
    app.add_message::<testing::E2EExtensionEffect>();
    let network = plugin.network;
    match network {
        testing::Network::Mainnet | testing::Network::DirectDial => {}
        testing::Network::LocalGateways => {
            tracing::error!(target: "testing", "local-gateways network is not implemented yet");
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
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::build_plugin;
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
}
