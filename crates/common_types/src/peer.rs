use serde::{Deserialize, Serialize};

use crate::PeerAddr;
use crate::PeerError;
use crate::PeerId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(bevy_reflect::Reflect))]
pub struct Peer {
    pub id: PeerId,
    pub addrs: Vec<PeerAddr>,
}

impl TryFrom<&str> for Peer {
    type Error = PeerError;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(PeerError::Empty);
        }
        let addr: multiaddr::Multiaddr = trimmed.parse()?;
        let ending = addr.iter().last();
        let multiaddr::Protocol::P2p(id) = ending.as_ref().ok_or(PeerError::MissingPeerId)? else {
            return Err(PeerError::MissingPeerId);
        };
        Ok(Self {
            id: PeerId::from(id.to_string()),
            addrs: vec![PeerAddr::from(trimmed)],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Peer;
    use crate::PeerError;
    use crate::PeerId;

    #[test]
    fn test_usage() {
        let peer = Peer {
            id: PeerId::from("me"),
            addrs: Vec::new(),
        };
        assert_eq!(peer.id.as_str(), "me");
    }

    fn friend_addr() -> String {
        let id = "12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC";
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
        assert!(matches!(Peer::try_from(""), Err(PeerError::Empty)));
        assert!(matches!(Peer::try_from("   "), Err(PeerError::Empty)));
        assert!(matches!(
            Peer::try_from("/ip4/127.0.0.1/tcp/4001"),
            Err(PeerError::MissingPeerId)
        ));
        assert!(matches!(
            Peer::try_from("not a multiaddr"),
            Err(PeerError::InvalidMultiaddr(_))
        ));
    }
}
