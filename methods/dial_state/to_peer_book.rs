use crate::direct_dial;

#[must_use]
pub fn to_peer_book(state: &direct_dial::state_machine::DialState) -> direct_dial::PeerBook {
    let mut book = direct_dial::PeerBook::default();
    for entry in &state.entries {
        book.push(direct_dial::KnownPeer {
            peer: entry.peer.clone(),
            kind: entry.kind,
        });
    }
    book
}

#[cfg(test)]
mod tests {
    use super::to_peer_book;
    use crate::direct_dial;
    use crate::discovery;
    use crate::identity;

    #[test]
    fn test_usage() {
        let mut book = direct_dial::PeerBook::default();
        assert!(book.add_friend(identity::Peer {
            id: identity::PeerId::from("alice"),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
        }));
        let state = direct_dial::state_machine::DialState::from_peer_book(
            &book,
            discovery::UnixTime::from(1),
        );
        assert_eq!(to_peer_book(&state), book);
    }
}
