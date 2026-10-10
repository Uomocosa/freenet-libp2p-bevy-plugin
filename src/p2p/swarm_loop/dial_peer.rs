use libp2p::swarm::dial_opts::{DialOpts, PeerCondition};

use crate::net_id;
use crate::p2p;

pub fn dial_peer<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    peer_id: &net_id::PeerId,
    addrs: &[net_id::PeerAddr],
    condition: PeerCondition,
) {
    let parsed: Vec<libp2p::Multiaddr> = addrs.iter().filter_map(|a| a.parse().ok()).collect();
    if parsed.is_empty() {
        return;
    }
    if let Ok(pid) = peer_id.parse::<libp2p::PeerId>() {
        let opts = DialOpts::peer_id(pid)
            .condition(condition)
            .addresses(parsed)
            .build();
        if let Err(e) = swarm.dial(opts) {
            event_tx
                .send(p2p::Event::Net(p2p::NetEvent::DialFailed {
                    peer_id: peer_id.clone(),
                    reason: e.to_string(),
                }))
                .ok();
        }
        return;
    }
    for addr in parsed {
        let _ = swarm.dial(addr);
    }
}

#[cfg(test)]
mod tests {
    use libp2p::identity::Keypair;
    use libp2p::swarm::dial_opts::PeerCondition;

    use super::dial_peer;
    use crate::net_id;
    use crate::p2p;

    fn swarm() -> libp2p::Swarm<p2p::Behaviour<u32>> {
        p2p::build_swarm::<u32>(Keypair::generate_ed25519(), p2p::MdnsMode::Disabled).unwrap()
    }

    #[tokio::test]
    async fn test_usage() {
        let mut swarm = swarm();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let peer = net_id::PeerId(libp2p::PeerId::random().to_string());
        let addrs = vec![net_id::PeerAddr::from("/ip4/127.0.0.1/tcp/9")];
        let gentle = PeerCondition::DisconnectedAndNotDialing;
        dial_peer(&mut swarm, &tx, &peer, &addrs, gentle);
        assert!(rx.try_recv().is_err());
        dial_peer(&mut swarm, &tx, &peer, &addrs, PeerCondition::Always);
        assert!(rx.try_recv().is_err());
        dial_peer(&mut swarm, &tx, &peer, &addrs, gentle);
        assert!(matches!(
            rx.try_recv(),
            Ok(p2p::Event::Net(p2p::NetEvent::DialFailed { .. }))
        ));
    }
}
