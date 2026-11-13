use crate::direct_dial;
use crate::identity;
use crate::time;

pub fn handle_peer_disconnected(
    state: &mut direct_dial::state_machine::DialState,
    id: &identity::PeerId,
    now: time::UnixTime,
) -> Vec<direct_dial::state_machine::Output> {
    state.connected.remove(id);
    state.dial_in_flight.remove(id);
    let current = state
        .entries
        .iter()
        .find(|entry| entry.peer.id == *id)
        .map(|entry| (entry.kind, entry.failed_attempts));
    if let Some((kind, failed_attempts)) = current {
        let wait = state.backoff_for(kind, failed_attempts);
        if let Some(entry) = state.entries.iter_mut().find(|entry| entry.peer.id == *id) {
            entry.next_attempt = time::Deadline::from_now_plus(now, wait);
        }
    }
    Vec::new()
}
#[cfg(test)]
mod tests {
    use super::handle_peer_disconnected;
    use crate::direct_dial;
    use crate::identity;
    use crate::time;

    #[test]
    fn test_usage() {
        let now = time::UnixTime::from_secs(5);
        let mut state = direct_dial::state_machine::DialState::default();
        let _ = state.handle_peer_connected(&identity::PeerId::from("alice"), now);
        let outputs = handle_peer_disconnected(&mut state, &identity::PeerId::from("alice"), now);
        assert_eq!(outputs, Vec::new());
        assert!(!state.connected.contains(&identity::PeerId::from("alice")));
    }
}
