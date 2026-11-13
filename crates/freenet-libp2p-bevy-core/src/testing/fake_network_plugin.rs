use std::marker::PhantomData;
use std::sync::atomic::{AtomicU32, Ordering};

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::{App, Plugin};

use crate::identity;
use crate::p2p;
use crate::testing;

static PEER_COUNTER: AtomicU32 = AtomicU32::new(0);

pub struct FakeNetworkPlugin<Payload> {
    pub network: testing::Network,
    pub orchestrator_port: Option<u16>,
    pub peer: identity::Peer,
    pub(crate) marker: PhantomData<Payload>,
}

impl<Payload> From<&testing::NetworkArgs> for FakeNetworkPlugin<Payload> {
    fn from(args: &testing::NetworkArgs) -> Self {
        let index = PEER_COUNTER.fetch_add(1, Ordering::Relaxed);
        let process = std::process::id();
        let peer = identity::Peer {
            id: identity::PeerId::from(format!("fake-peer-{process}-{index}")),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
        };
        Self {
            network: args.network,
            orchestrator_port: args.orchestrator_port,
            peer,
            marker: PhantomData,
        }
    }
}

#[rustfmt::skip]
impl<Payload: p2p::traits::Message> Plugin for FakeNetworkPlugin<Payload> {
    fn build(&self, app: &mut App) { self.build_plugin(app); }
}

#[atomic_delegates]
impl<Payload: p2p::traits::Message> FakeNetworkPlugin<Payload> {
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
        let plugin = super::FakeNetworkPlugin::<u8>::from(&args);
        assert!(matches!(plugin.network, testing::Network::Mainnet));
    }
}
