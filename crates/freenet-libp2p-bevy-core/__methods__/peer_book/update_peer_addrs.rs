use crate::direct_dial;
use crate::identity;

pub fn update_peer_addrs(book: &mut direct_dial::PeerBook, peer: &identity::Peer) -> bool {
    if let Some(entry) = book.iter_mut().find(|entry| entry.peer.id == peer.id) {
        let changed = entry.peer.addrs != peer.addrs;
        if changed {
            entry.peer.addrs.clone_from(&peer.addrs);
        }
        changed
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::update_peer_addrs;
    use crate::direct_dial;
    use crate::identity;

    #[test]
    fn test_usage() {
        let mut book = direct_dial::PeerBook::default();
        let alice = identity::Peer {
            id: identity::PeerId::from("alice"),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
        };
        book.push(direct_dial::KnownPeer {
            peer: alice.clone(),
            kind: direct_dial::KnownPeerKind::Friend,
        });
        assert!(!update_peer_addrs(&mut book, &alice));
        let refreshed = identity::Peer {
            id: identity::PeerId::from("alice"),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/5001")],
        };
        assert!(update_peer_addrs(&mut book, &refreshed));
        assert_eq!(book[0].peer.addrs, refreshed.addrs);
        let stranger = identity::Peer {
            id: identity::PeerId::from("mallory"),
            addrs: Vec::new(),
        };
        assert!(!update_peer_addrs(&mut book, &stranger));
    }
}
