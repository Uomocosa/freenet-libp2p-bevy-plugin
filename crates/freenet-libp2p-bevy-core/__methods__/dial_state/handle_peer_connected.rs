use crate::direct_dial;
use crate::identity;

pub fn handle_peer_connected(
    state: &mut direct_dial::state_machine::DialState,
    id: &identity::PeerId,
) -> Vec<direct_dial::state_machine::Output> {
    state.connected.insert(id.clone());
    state.dial_in_flight.remove(id);
    if let Some(entry) = state.entries.iter_mut().find(|entry| entry.peer.id == *id) {
        entry.failed_attempts = 0;
    }
    Vec::new()
}
#[cfg(test)]
mod tests {
    use super::handle_peer_connected;
    use crate::direct_dial;
    use crate::identity;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut state = direct_dial::state_machine::DialState::default();
        let outputs = handle_peer_connected(&mut state, &identity::PeerId::from("alice"));
        assert_eq!(outputs, Vec::new());
        assert!(state.connected.contains(&identity::PeerId::from("alice")));
        assert_eq!(state.entries, Vec::new());
    }

    #[test]
    fn new_peer_without_addresses_is_not_written_to_the_book() {
        let mut state = direct_dial::state_machine::DialState::default();
        let outputs = handle_peer_connected(&mut state, &identity::PeerId::from("bob"));
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.entries, Vec::new());
        assert!(state.connected.contains(&identity::PeerId::from("bob")));
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
        let outputs = handle_peer_connected(&mut state, &identity::PeerId::from("alice"));
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.entries.len(), 1);
        assert_eq!(state.entries[0].kind, direct_dial::KnownPeerKind::Friend);
    }
}
