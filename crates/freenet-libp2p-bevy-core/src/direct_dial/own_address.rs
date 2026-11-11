use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnAddress {
    pub id: identity::PeerId,
    pub short: String,
    pub full: Option<String>,
}

impl OwnAddress {
    #[must_use]
    pub fn from_own_peer(peer: &identity::Peer, observed: &[identity::PeerAddr]) -> Self {
        let dialable = identity::PeerAddr::dialable(peer.addrs.clone());
        let full = observed
            .iter()
            .chain(&dialable)
            .next()
            .map(|addr| format!("{addr}/p2p/{}", peer.id));
        Self {
            id: peer.id.clone(),
            short: identity::ShortPeerId::from(&peer.id).to_string(),
            full,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OwnAddress;
    use crate::identity;

    #[test]
    fn test_usage() {
        let peer = identity::Peer {
            id: identity::PeerId::from("12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC"),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
        };
        let address = OwnAddress::from_own_peer(&peer, &[]);
        assert_eq!(
            address.full.as_deref(),
            Some(
                "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC"
            )
        );
        assert_eq!(address.short, "12D3Koo...99RC");
        assert_eq!(
            address.id,
            identity::PeerId::from("12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC")
        );
    }

    #[test]
    fn without_an_address_only_the_id_is_known() {
        let peer = identity::Peer {
            id: identity::PeerId::from("12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC"),
            addrs: Vec::new(),
        };
        let address = OwnAddress::from_own_peer(&peer, &[]);
        assert_eq!(address.full, None);
        assert_eq!(address.short, "12D3Koo...99RC");
    }

    #[test]
    fn an_observed_address_wins_over_the_own_addrs() {
        let peer = identity::Peer {
            id: identity::PeerId::from("peer-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            addrs: vec![identity::PeerAddr::from("/ip4/10.0.0.5/tcp/4001")],
        };
        let observed = vec![identity::PeerAddr::from("/ip4/1.2.3.4/tcp/4001")];
        let address = OwnAddress::from_own_peer(&peer, &observed);
        assert_eq!(
            address.full.as_deref(),
            Some("/ip4/1.2.3.4/tcp/4001/p2p/peer-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
    }
}
