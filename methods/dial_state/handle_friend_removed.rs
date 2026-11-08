use crate::direct_dial;
use crate::identity;

pub fn handle_friend_removed(
    state: &mut direct_dial::state_machine::DialState,
    id: &identity::PeerId,
) -> Vec<direct_dial::state_machine::Output> {
    state.entries.retain(|entry| entry.peer.id != *id);
    state.dial_in_flight.remove(id);
    Vec::new()
}
#[cfg(test)]
mod tests {
    use super::handle_friend_removed;
    use crate::direct_dial;
    use crate::identity;
    use crate::time;

    #[test]
    fn test_usage() {
        let now = time::UnixTime::from(5);
        let mut state = direct_dial::state_machine::DialState::default();
        let peer = identity::Peer {
            id: identity::PeerId::from("alice"),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
        };
        let _ = state.handle_friend_added(peer, now);
        assert_eq!(state.entries.len(), 1);
        let outputs = handle_friend_removed(&mut state, &identity::PeerId::from("alice"));
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.entries, Vec::new());
    }
}
