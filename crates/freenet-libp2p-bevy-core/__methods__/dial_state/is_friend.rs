use crate::direct_dial;
use crate::identity;

pub fn is_friend(state: &direct_dial::state_machine::DialState, id: &identity::PeerId) -> bool {
    state
        .entries
        .iter()
        .any(|entry| entry.peer.id == *id && entry.kind == direct_dial::KnownPeerKind::Friend)
}

#[cfg(test)]
mod tests {
    use super::is_friend;
    use crate::direct_dial;
    use crate::identity;
    use crate::time;

    #[test]
    fn test_usage() {
        let alice = identity::PeerId::from("alice");
        let mut book = direct_dial::PeerBook::default();
        book.push(direct_dial::KnownPeer {
            peer: identity::Peer {
                id: alice.clone(),
                addrs: Vec::new(),
            },
            kind: direct_dial::KnownPeerKind::Friend,
        });
        let state =
            direct_dial::state_machine::DialState::from_peer_book(&book, time::UnixTime::EPOCH);
        assert!(is_friend(&state, &alice));
        assert!(!is_friend(&state, &identity::PeerId::from("mallory")));
    }
}
