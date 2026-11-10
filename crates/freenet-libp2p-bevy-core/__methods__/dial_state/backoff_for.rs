use std::time::Duration;

use crate::direct_dial;

#[must_use]
pub fn backoff_for(
    state: &direct_dial::state_machine::DialState,
    kind: direct_dial::KnownPeerKind,
    failed_attempts: u32,
) -> Duration {
    let _ = state;
    let capped = failed_attempts.min(6);
    let factor = 2_u32.pow(capped);
    let initial = match kind {
        direct_dial::KnownPeerKind::Friend => direct_dial::state_machine::FRIEND_INITIAL_BACKOFF,
        direct_dial::KnownPeerKind::PreviouslyConnected => {
            direct_dial::state_machine::KNOWN_INITIAL_BACKOFF
        }
    };
    let max = match kind {
        direct_dial::KnownPeerKind::Friend => direct_dial::state_machine::FRIEND_MAX_BACKOFF,
        direct_dial::KnownPeerKind::PreviouslyConnected => {
            direct_dial::state_machine::KNOWN_MAX_BACKOFF
        }
    };
    let grown = initial.saturating_mul(factor);
    grown.min(max)
}

#[cfg(test)]
mod tests {
    use super::backoff_for;
    use crate::direct_dial;

    #[test]
    fn test_usage() {
        let state = direct_dial::state_machine::DialState::default();
        let wait = backoff_for(&state, direct_dial::KnownPeerKind::Friend, 0);
        assert_eq!(wait, direct_dial::state_machine::FRIEND_INITIAL_BACKOFF);
    }
}
