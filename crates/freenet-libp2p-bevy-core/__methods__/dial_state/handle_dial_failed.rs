use crate::direct_dial;
use crate::identity;
use crate::time;

pub fn handle_dial_failed(
    state: &mut direct_dial::state_machine::DialState,
    id: &identity::PeerId,
    now: time::UnixTime,
) -> Vec<direct_dial::state_machine::Output> {
    state.dial_in_flight.remove(id);
    if state.connected.contains(id) {
        return Vec::new();
    }
    let found = state
        .entries
        .iter_mut()
        .find(|entry| entry.peer.id == *id)
        .map(|entry| {
            entry.failed_attempts = entry.failed_attempts.saturating_add(1);
            (entry.kind, entry.failed_attempts)
        });
    let Some((kind, failed_attempts)) = found else {
        tracing::debug!(target: "direct_dial", peer = %id.as_str(), "dial failed for unknown peer");
        return Vec::new();
    };
    if kind == direct_dial::KnownPeerKind::PreviouslyConnected
        && failed_attempts >= direct_dial::state_machine::KNOWN_MAX_FAILURES
    {
        let dead = id.clone();
        state.entries.retain(|entry| entry.peer.id != dead);
        return vec![direct_dial::state_machine::Output::SavePeerBook(
            state.to_peer_book(),
        )];
    }
    let wait = state.backoff_for(kind, failed_attempts);
    if let Some(entry) = state.entries.iter_mut().find(|entry| entry.peer.id == *id)
        && let Some(next) = now.checked_add(wait)
    {
        entry.next_attempt_at = next;
    }
    Vec::new()
}
#[cfg(test)]
mod tests {
    use super::handle_dial_failed;
    use crate::direct_dial;
    use crate::identity;
    use crate::time;

    #[test]
    fn test_usage() {
        let now = time::UnixTime::from_secs(5);
        let mut state = direct_dial::state_machine::DialState::default();
        let peer = identity::Peer {
            id: identity::PeerId::from("alice"),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
        };
        let _ = state.handle_friend_added(peer.clone(), now);
        state.dial_in_flight.insert(peer.id.clone());
        let outputs = handle_dial_failed(&mut state, &peer.id, now);
        assert_eq!(outputs, Vec::new());
        assert!(!state.dial_in_flight.contains(&peer.id));
    }

    #[test]
    fn unknown_id_logs_and_returns_no_output() {
        let now = time::UnixTime::from_secs(5);
        let mut state = direct_dial::state_machine::DialState::default();
        let outputs = handle_dial_failed(&mut state, &identity::PeerId::from("ghost"), now);
        assert_eq!(outputs, Vec::new());
    }
}
