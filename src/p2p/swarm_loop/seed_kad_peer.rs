use libp2p::kad;

use crate::net_id;
use crate::p2p;

pub fn seed_kad_peer<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    peer_id: &net_id::PeerId,
    addrs: &[net_id::PeerAddr],
) {
    let Ok(pid) = peer_id.parse::<libp2p::PeerId>() else {
        return;
    };
    let mut added = false;
    for addr in addrs {
        if let Ok(ma) = addr.parse::<libp2p::Multiaddr>()
            && swarm.behaviour_mut().kademlia.add_address(&pid, ma) == kad::RoutingUpdate::Success
        {
            added = true;
        }
    }
    if added {
        let _ = swarm.behaviour_mut().kademlia.bootstrap();
    }
}

#[cfg(test)]
mod tests {
    use libp2p::identity::Keypair;

    use super::seed_kad_peer;
    use crate::net_id;
    use crate::p2p;

    fn swarm() -> libp2p::Swarm<p2p::Behaviour<u32>> {
        p2p::build_swarm::<u32>(Keypair::generate_ed25519(), p2p::MdnsMode::Disabled).unwrap()
    }

    #[tokio::test]
    async fn test_usage() {
        let mut swarm = swarm();
        let peer = libp2p::PeerId::random();
        let addrs = vec![
            net_id::PeerAddr::from("/ip4/10.0.0.2/tcp/4001"),
            net_id::PeerAddr::from("not-an-addr"),
        ];
        seed_kad_peer(&mut swarm, &net_id::PeerId(peer.to_string()), &addrs);
        let known = swarm.behaviour_mut().kademlia.kbuckets().any(|bucket| {
            bucket
                .iter()
                .any(|entry| *entry.node.key.preimage() == peer)
        });
        assert!(known);
    }
}
