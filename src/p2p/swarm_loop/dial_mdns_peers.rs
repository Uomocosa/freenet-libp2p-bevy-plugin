use libp2p::swarm::dial_opts::{DialOpts, PeerCondition};

use crate::p2p;

pub fn dial_mdns_peers<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    mode: p2p::TransportMode,
    peers: Vec<(libp2p::PeerId, libp2p::Multiaddr)>,
) {
    for (peer, addr) in peers {
        if !p2p::matches_transport(mode, &addr) {
            continue;
        }
        let opts = DialOpts::peer_id(peer)
            .condition(PeerCondition::DisconnectedAndNotDialing)
            .addresses(vec![addr])
            .build();
        if let Err(e) = swarm.dial(opts) {
            tracing::debug!(target: "p2p", error = %e, "mdns dial skipped");
        }
    }
}

// no test_usage necessary
