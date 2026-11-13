use libp2p::identify;

use crate::identity;
use crate::p2p;

pub fn handle_identify<Payload: p2p::traits::Message>(
    state: &p2p::SwarmLoopState<Payload>,
    peer_id: &libp2p::PeerId,
    info: &identify::Info,
) {
    state
        .event_tx
        .send(p2p::Event::Libp2p(p2p::Libp2pEvent::ObservedAddr(
            identity::PeerAddr(info.observed_addr.to_string()),
        )))
        .ok();
    let reported: Vec<identity::PeerAddr> = info
        .listen_addrs
        .iter()
        .map(|addr| identity::PeerAddr::from(addr.to_string()))
        .collect();
    let dialable = identity::PeerAddr::dialable(reported);
    if !dialable.is_empty() {
        state
            .event_tx
            .send(p2p::Event::Libp2p(p2p::Libp2pEvent::PeerListenAddrs {
                peer_id: identity::PeerId::from(peer_id.to_string()),
                addrs: dialable,
            }))
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use libp2p::identify;

    use super::handle_identify;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let (state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        let info = identify::Info {
            public_key: libp2p::identity::Keypair::generate_ed25519().public(),
            protocol_version: "test/1".to_string(),
            agent_version: "test".to_string(),
            listen_addrs: Vec::new(),
            protocols: Vec::new(),
            observed_addr: "/ip4/1.2.3.4/tcp/4001".parse().expect("multiaddr"),
            signed_peer_record: None,
        };
        handle_identify(&state, &libp2p::PeerId::random(), &info);
        assert!(matches!(
            rx.try_recv(),
            Ok(p2p::Event::Libp2p(p2p::Libp2pEvent::ObservedAddr(addr)))
                if addr == identity::PeerAddr::from("/ip4/1.2.3.4/tcp/4001")
        ));
    }
}
