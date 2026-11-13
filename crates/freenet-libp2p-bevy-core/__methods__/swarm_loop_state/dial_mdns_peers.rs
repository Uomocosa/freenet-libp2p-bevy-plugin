use crate::p2p;
use libp2p::swarm::dial_opts::{DialOpts, PeerCondition};

pub fn dial_mdns_peers<Payload: p2p::traits::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    peers: Vec<(libp2p::PeerId, libp2p::Multiaddr)>,
) {
    for (peer, addr) in peers {
        if !state.mode.matches_transport(&addr) {
            continue;
        }
        let opts = DialOpts::peer_id(peer)
            .condition(PeerCondition::DisconnectedAndNotDialing)
            .addresses(vec![addr])
            .build();
        if let Err(e) = state.swarm.dial(opts) {
            tracing::debug!(target: "p2p", error = %e, "mdns dial skipped");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::dial_mdns_peers;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, _rx) = p2p::SwarmLoopState::<u32>::for_test();
        dial_mdns_peers(&mut state, Vec::new());
        assert_eq!(state.swarm.connected_peers().count(), 0);
    }
}
