use libp2p::swarm::dial_opts::{DialOpts, PeerCondition};

use crate::identity;
use crate::p2p;

pub fn dial_peer<Payload: p2p::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    peer_id: &identity::PeerId,
    addrs: &[identity::PeerAddr],
    condition: PeerCondition,
) {
    let multiaddrs: Vec<libp2p::Multiaddr> = addrs.iter().filter_map(|a| a.parse().ok()).collect();
    if multiaddrs.is_empty() {
        return;
    }
    if let Ok(libp2p_id) = peer_id.parse::<libp2p::PeerId>() {
        let opts = DialOpts::peer_id(libp2p_id)
            .condition(condition)
            .addresses(multiaddrs)
            .build();
        if let Err(e) = state.swarm.dial(opts) {
            state
                .event_tx
                .send(p2p::Event::Net(p2p::NetEvent::DialFailed {
                    peer_id: peer_id.clone(),
                    reason: e.to_string(),
                }))
                .ok();
        }
        return;
    }
    for addr in multiaddrs {
        let _ = state.swarm.dial(addr);
    }
}

#[cfg(test)]
mod tests {
    use super::dial_peer;
    use libp2p::swarm::dial_opts::PeerCondition;

    use crate::identity;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        let peer = identity::PeerId(libp2p::PeerId::random().to_string());
        let addrs = vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/9")];
        let gentle = PeerCondition::DisconnectedAndNotDialing;
        dial_peer(&mut state, &peer, &addrs, gentle);
        assert!(rx.try_recv().is_err());
        dial_peer(&mut state, &peer, &addrs, PeerCondition::Always);
        assert!(rx.try_recv().is_err());
        dial_peer(&mut state, &peer, &addrs, gentle);
        assert!(matches!(
            rx.try_recv(),
            Ok(p2p::Event::Net(p2p::NetEvent::DialFailed { .. }))
        ));
    }
}
