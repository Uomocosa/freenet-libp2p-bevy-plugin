use std::marker::PhantomData;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::{App, Plugin};

use crate::identity;
use crate::p2p;
use crate::testing;

pub struct FakeNetworkPlugin<Payload> {
    pub network: testing::Network,
    pub orchestrator_port: Option<u16>,
    pub peer: identity::Peer,
    pub(crate) marker: PhantomData<Payload>,
}

#[atomic_delegates]
impl<Payload> From<&testing::NetworkArgs> for FakeNetworkPlugin<Payload> {
    fn from(args: &testing::NetworkArgs) -> Self {}
}

impl<Payload: p2p::traits::Message> Plugin for FakeNetworkPlugin<Payload> {
    fn build(&self, app: &mut App) {
        app.add_message::<testing::E2EExtensionEffect>();
        match self.network {
            testing::Network::Mainnet | testing::Network::DirectDial => {}
            testing::Network::LocalGateways => {
                tracing::error!(target: "testing", "local-gateways network is not implemented yet");
            }
            testing::Network::Fake => {
                let Some(port) = self.orchestrator_port else {
                    tracing::error!(
                        target: "testing",
                        "fake network needs --orchestrator-port"
                    );
                    return;
                };
                app.insert_resource(testing::FakeTaskHandles::default());
                app.insert_resource(p2p::SwarmTaskOverride::<Payload> {
                    orchestrator_port: port,
                    peer: self.peer.clone(),
                    marker: PhantomData,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::p2p;
    use crate::testing;

    #[test]
    fn test_usage() {
        let args = testing::NetworkArgs {
            network: testing::Network::Mainnet,
            orchestrator_port: None,
        };
        let plugin = super::FakeNetworkPlugin::<u8>::from(&args);
        assert!(matches!(plugin.network, testing::Network::Mainnet));
    }

    #[test]
    fn mainnet_build_adds_no_swarm_override() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let args = testing::NetworkArgs {
            network: testing::Network::Mainnet,
            orchestrator_port: None,
        };
        let plugin = testing::FakeNetworkPlugin::<u8>::from(&args);
        app.add_plugins(plugin);
        assert!(
            app.world()
                .get_resource::<p2p::SwarmTaskOverride<u8>>()
                .is_none()
        );
    }
}
