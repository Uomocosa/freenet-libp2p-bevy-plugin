use std::time::Duration;

use crate::identity;
use crate::p2p;
use crate::spectator;
use crate::time;

#[must_use]
pub fn spectating_state(
    me: Option<&str>,
    started_at: time::UnixTime,
    lease_interval: Duration,
) -> spectator::state_machine::SpectatorState<u8, Vec<u8>> {
    let mut state = spectator::state_machine::SpectatorState::new(identity::GameId::from("game"));
    state.me = me.map(identity::PeerId::from);
    state.start_spectating(
        &identity::ZoneName::from("zone"),
        p2p::HistoryKeyPrefix::from("prefix"),
        started_at,
        lease_interval,
    );
    state
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::p2p;
    use crate::spectator;
    use crate::time;

    use super::spectating_state;

    #[test]
    fn test_usage() {
        let state = spectating_state(
            Some("me"),
            time::UnixTime::from_secs(10),
            std::time::Duration::from_secs(1),
        );
        assert_eq!(state.me, Some(identity::PeerId::from("me")));
        assert_eq!(state.zone, Some(identity::ZoneName::from("zone")));
        assert_eq!(
            state.history_prefix,
            Some(p2p::HistoryKeyPrefix::from("prefix"))
        );
        assert_eq!(
            state.phase,
            spectator::state_machine::SpectatePhase::FetchingSnapshot
        );
    }
}
