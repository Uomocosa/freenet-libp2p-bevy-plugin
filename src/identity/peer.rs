use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};
use tokio::sync::watch::Receiver;

use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct Peer {
    pub id: identity::PeerId,
    pub addrs: Vec<identity::PeerAddr>,
}

impl TryFrom<&str> for Peer {
    type Error = identity::PeerError;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(identity::PeerError::Empty);
        }
        let addr: libp2p::Multiaddr = trimmed.parse()?;
        let ending = addr.iter().last();
        let libp2p::multiaddr::Protocol::P2p(id) =
            ending.as_ref().ok_or(identity::PeerError::MissingPeerId)?
        else {
            return Err(identity::PeerError::MissingPeerId);
        };
        Ok(Self {
            id: identity::PeerId::from(id.to_string()),
            addrs: vec![identity::PeerAddr::from(trimmed)],
        })
    }
}

#[atomic_delegates]
impl Peer {
    pub async fn wait_ready(ready: &mut Receiver<Option<Peer>>) -> Option<Peer> {}
}

#[cfg(test)]
mod tests {
    use super::Peer;
    use crate::identity;

    #[test]
    fn test_usage() {
        let peer = Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        };
        assert_eq!(peer.id.as_str(), "me");
    }

    fn friend_addr() -> String {
        let id = libp2p::PeerId::random().to_string();
        format!("/ip4/127.0.0.1/tcp/4001/p2p/{id}")
    }

    #[test]
    fn parses_a_multiaddr_ending_in_a_peer_id() {
        let text = friend_addr();
        let peer = Peer::try_from(text.as_str()).expect("good address");
        assert_eq!(peer.addrs.len(), 1);
        assert!(peer.id.as_str().len() > 10);
    }

    #[test]
    fn rejects_bad_addresses() {
        assert!(matches!(
            Peer::try_from(""),
            Err(identity::PeerError::Empty)
        ));
        assert!(matches!(
            Peer::try_from("   "),
            Err(identity::PeerError::Empty)
        ));
        assert!(matches!(
            Peer::try_from("/ip4/127.0.0.1/tcp/4001"),
            Err(identity::PeerError::MissingPeerId)
        ));
        assert!(matches!(
            Peer::try_from("not a multiaddr"),
            Err(identity::PeerError::InvalidMultiaddr(_))
        ));
    }
}
