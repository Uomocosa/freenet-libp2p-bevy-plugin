use crate::direct_dial;
use crate::identity;
use crate::time;

pub fn handle_peer_connected(
    state: &mut direct_dial::state_machine::DialState,
    id: &identity::PeerId,
    now: time::UnixTime,
) -> Vec<direct_dial::state_machine::Output> {
    state.connected.insert(id.clone());
    state.dial_in_flight.remove(id);
    if let Some(entry) = state.entries.iter_mut().find(|entry| entry.peer.id == *id) {
        entry.failed_attempts = 0;
        return Vec::new();
    }
    {
        state
            .entries
            .push(direct_dial::state_machine::DialKnownPeer {
                peer: identity::Peer {
                    id: id.clone(),
                    addrs: Vec::new(),
                },
                kind: direct_dial::KnownPeerKind::PreviouslyConnected,
                failed_attempts: 0,
                next_attempt: time::Deadline::at(now),
            });
        vec![direct_dial::state_machine::Output::SavePeerBook(
            state.to_peer_book(),
        )]
    }
}
#[cfg(test)]
mod tests {
    use super::handle_peer_connected;
    use crate::direct_dial;
    use crate::identity;
    use crate::time;

    #[test]
    fn test_usage() {
        let now = time::UnixTime::from_secs(5);
        let mut state = direct_dial::state_machine::DialState::default();
        let outputs = handle_peer_connected(&mut state, &identity::PeerId::from("alice"), now);
        assert_eq!(outputs.len(), 1);
        assert!(state.connected.contains(&identity::PeerId::from("alice")));
        assert_eq!(state.entries.len(), 1);
    }

    #[test]
    fn new_entry_saves_the_book_once() {
        let now = time::UnixTime::from_secs(5);
        let mut state = direct_dial::state_machine::DialState::default();
        let outputs = handle_peer_connected(&mut state, &identity::PeerId::from("bob"), now);
        let [direct_dial::state_machine::Output::SavePeerBook(book)] = outputs.as_slice() else {
            panic!("expected one SavePeerBook, got {outputs:?}");
        };
        assert_eq!(book.len(), 1);
        assert_eq!(
            book[0].kind,
            direct_dial::KnownPeerKind::PreviouslyConnected
        );
    }

    #[test]
    fn existing_entry_emits_no_save() {
        let now = time::UnixTime::from_secs(5);
        let mut book = direct_dial::PeerBook::default();
        assert!(book.add_friend(identity::Peer {
            id: identity::PeerId::from("alice"),
            addrs: Vec::new(),
        }));
        let mut state = direct_dial::state_machine::DialState::from_peer_book(&book, now);
        let outputs = handle_peer_connected(&mut state, &identity::PeerId::from("alice"), now);
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.entries.len(), 1);
        assert_eq!(state.entries[0].kind, direct_dial::KnownPeerKind::Friend);
    }
}
