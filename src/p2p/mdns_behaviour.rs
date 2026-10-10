use libp2p::mdns;
use libp2p::swarm::behaviour::toggle::Toggle;

use crate::p2p;

#[must_use]
pub fn mdns_behaviour(
    mode: p2p::MdnsMode,
    peer_id: libp2p::PeerId,
) -> Toggle<mdns::tokio::Behaviour> {
    match mode {
        p2p::MdnsMode::Disabled => Toggle::from(None),
        p2p::MdnsMode::Enabled => {
            Toggle::from(mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id).ok())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mdns_behaviour;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let peer_id = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        assert!(!mdns_behaviour(p2p::MdnsMode::Disabled, peer_id).is_enabled());
    }
}
