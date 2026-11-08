use crate::direct_dial;
use crate::time;

#[must_use]
pub fn from_peer_book(
    book: &direct_dial::PeerBook,
    now: time::UnixTime,
) -> direct_dial::state_machine::DialState {
    let mut state = direct_dial::state_machine::DialState::default();
    for entry in book.iter() {
        state
            .entries
            .push(direct_dial::state_machine::DialKnownPeer {
                peer: entry.peer.clone(),
                kind: entry.kind,
                failed_attempts: 0,
                next_attempt_at: now,
            });
    }
    state
}
#[cfg(test)]
mod tests {
    use super::from_peer_book;
    use crate::direct_dial;
    use crate::identity;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut book = direct_dial::PeerBook::default();
        assert!(book.add_friend(identity::Peer {
            id: identity::PeerId::from("alice"),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
        }));
        let state = from_peer_book(&book, time::UnixTime::from(7));
        assert_eq!(state.entries.len(), 1);
        assert_eq!(state.entries[0].next_attempt_at, time::UnixTime::from(7));
    }
}
