use std::sync::atomic::{AtomicU32, Ordering};

use crate::identity;
use crate::testing;

pub fn from_network_args<Payload>(
    args: &testing::NetworkArgs,
) -> testing::FakeNetworkPlugin<Payload> {
    static PEER_COUNTER: AtomicU32 = AtomicU32::new(0);
    let index = PEER_COUNTER.fetch_add(1, Ordering::Relaxed);
    let process = std::process::id();
    let peer = identity::Peer {
        id: identity::PeerId::from(format!("fake-peer-{process}-{index}")),
        addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
    };
    testing::FakeNetworkPlugin {
        network: args.network,
        orchestrator_port: args.orchestrator_port,
        peer,
        marker: std::marker::PhantomData,
    }
}

#[cfg(test)]
mod tests {
    use super::from_network_args;
    use crate::testing;

    #[test]
    fn test_usage() {
        let args = testing::NetworkArgs {
            network: testing::Network::Mainnet,
            orchestrator_port: None,
        };
        let plugin = from_network_args::<u8>(&args);
        assert!(matches!(plugin.network, testing::Network::Mainnet));
    }
}
