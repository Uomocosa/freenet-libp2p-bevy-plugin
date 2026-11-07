use crate::direct_dial;
use crate::discovery;
use crate::identity;

pub fn dial_due(
    state: &mut direct_dial::state_machine::DialState,
    now: discovery::UnixTime,
) -> Vec<direct_dial::state_machine::Output> {
    let mut due_ids: Vec<identity::PeerId> = Vec::new();
    for entry in &state.entries {
        let is_connected = state.connected.contains(&entry.peer.id);
        let is_in_flight = state.dial_in_flight.contains(&entry.peer.id);
        let has_addrs = !entry.peer.addrs.is_empty();
        let is_due = entry.next_attempt_at <= now;
        if !is_connected && !is_in_flight && has_addrs && is_due {
            due_ids.push(entry.peer.id.clone());
        }
    }
    let mut outputs: Vec<direct_dial::state_machine::Output> = Vec::new();
    for id in due_ids {
        let found = state.entries.iter().find(|entry| entry.peer.id == id);
        if let Some(entry) = found {
            state.dial_in_flight.insert(id);
            outputs.push(direct_dial::state_machine::Output::Dial(entry.peer.clone()));
        }
    }
    outputs
}

#[cfg(test)]
mod tests {
    use super::dial_due;
    use crate::direct_dial;
    use crate::discovery;
    use crate::identity;

    fn friend_state(now: discovery::UnixTime) -> direct_dial::state_machine::DialState {
        let mut book = direct_dial::PeerBook::default();
        assert!(book.add_friend(identity::Peer {
            id: identity::PeerId::from("alice"),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
        }));
        direct_dial::state_machine::DialState::from_peer_book(&book, now)
    }

    #[test]
    fn test_usage() {
        let now = discovery::UnixTime::from(10);
        let mut state = friend_state(now);
        let outputs = dial_due(&mut state, now);
        assert_eq!(outputs.len(), 1);
        assert!(
            state
                .dial_in_flight
                .contains(&identity::PeerId::from("alice"))
        );
    }
}
