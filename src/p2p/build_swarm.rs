use libp2p::identity::Keypair;
use libp2p::kad::store::MemoryStore;
use libp2p::{
    StreamProtocol, dcutr, gossipsub, identify, kad, noise, ping, relay, request_response, tcp,
    yamux,
};

use crate::p2p;

pub fn build_swarm<T: p2p::Message>(
    keypair: Keypair,
    mdns: p2p::MdnsMode,
) -> Result<libp2p::Swarm<p2p::Behaviour<T>>, String> {
    let peer_id = keypair.public().to_peer_id();
    let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|e| e.to_string())?
        .with_quic()
        .with_dns()
        .map_err(|e| e.to_string())?
        .with_relay_client(noise::Config::new, yamux::Config::default)
        .map_err(|e| e.to_string())?
        .with_behaviour(
            |kp,
             relay_client|
             -> Result<p2p::Behaviour<T>, Box<dyn std::error::Error + Send + Sync>> {
                let mut kad_cfg = kad::Config::default();
                let Some(replication) = std::num::NonZeroUsize::new(8) else {
                    return Err("invalid replication factor".into());
                };
                kad_cfg.set_replication_factor(replication);
                let store = MemoryStore::new(kp.public().to_peer_id());
                let mut kademlia = kad::Behaviour::with_config(peer_id, store, kad_cfg);
                kademlia.set_mode(Some(kad::Mode::Server));
                let rr = request_response::Behaviour::with_codec(
                    p2p::MessageCodec::<T>::default(),
                    [(
                        StreamProtocol::new("/lobby/1.0.0"),
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                );
                let exchange = request_response::Behaviour::with_codec(
                    p2p::MessageCodec::<Vec<u8>>::default(),
                    [(
                        StreamProtocol::new("/lobby/pex/1.0.0"),
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                );
                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(kp.clone()),
                    gossipsub::Config::default(),
                )
                .map_err(|e| format!("gossipsub: {e}"))?;
                Ok(p2p::Behaviour {
                    request_response: rr,
                    exchange,
                    kademlia,
                    identify: identify::Behaviour::new(identify::Config::new(
                        "/lobby/id/1.0.0".to_string(),
                        kp.public(),
                    )),
                    ping: ping::Behaviour::default(),
                    gossipsub,
                    relay: relay::Behaviour::new(peer_id, relay::Config::default()),
                    relay_client,
                    dcutr: dcutr::Behaviour::new(peer_id),
                    mdns: p2p::mdns_behaviour(mdns, peer_id),
                })
            },
        )
        .map_err(|e| e.to_string())?
        .with_swarm_config(|cfg| {
            cfg.with_idle_connection_timeout(std::time::Duration::from_secs(u64::MAX))
        })
        .build();
    Ok(swarm)
}

#[cfg(test)]
mod tests {
    use libp2p::identity::Keypair;

    use super::build_swarm;
    use crate::p2p;
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let kp = Keypair::generate_ed25519();
        let swarm = build_swarm::<Dummy>(kp, p2p::MdnsMode::Disabled);
        assert!(swarm.is_ok());
    }
}
