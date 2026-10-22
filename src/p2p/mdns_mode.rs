use libp2p::mdns;
use libp2p::swarm::behaviour::toggle::Toggle;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MdnsMode {
    Enabled,
    Disabled,
}

#[rustfmt::skip]
impl MdnsMode {
    #[must_use]
    pub fn mdns_behaviour(&self, peer_id: libp2p::PeerId) -> Toggle<mdns::tokio::Behaviour> {
        match self {
            Self::Disabled => Toggle::from(None),
            Self::Enabled => {
                Toggle::from(mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id).ok())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MdnsMode;

    #[test]
    fn test_usage() {
        assert_eq!(MdnsMode::Enabled, MdnsMode::Enabled);
    }
}
