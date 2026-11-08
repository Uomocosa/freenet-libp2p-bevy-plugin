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
    match state.entries.iter_mut().find(|entry| entry.peer.id == *id) {
        Some(entry) => {
            entry.failed_attempts = 0;
        }
        None => {
            state
                .entries
                .push(direct_dial::state_machine::DialKnownPeer {
                    peer: identity::Peer {
                        id: id.clone(),
                        addrs: Vec::new(),
                    },
                    kind: direct_dial::KnownPeerKind::PreviouslyConnected,
                    failed_attempts: 0,
                    next_attempt_at: now,
                });
        }
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
        let now = time::UnixTime::from(5);
        let mut state = direct_dial::state_machine::DialState::default();
        let outputs = handle_peer_connected(&mut state, &identity::PeerId::from("alice"), now);
        assert_eq!(outputs, Vec::new());
        assert!(state.connected.contains(&identity::PeerId::from("alice")));
        assert_eq!(state.entries.len(), 1);
    }
}
