use crate::identity;
use crate::spectator;
use crate::time;

pub fn handle_local_peer<Op, Snap>(
    state: &mut spectator::state_machine::SpectatorState<Op, Snap>,
    peer: Option<identity::PeerId>,
    now: time::UnixTime,
    lease_interval: std::time::Duration,
) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    if state.me == peer {
        return Vec::new();
    }
    state.me = peer;
    if state.zone.is_some() {
        state.publish_lease(now, lease_interval)
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::handle_local_peer;
    use crate::identity;
    use crate::p2p;
    use crate::spectator;
    use crate::time;

    fn state() -> spectator::state_machine::SpectatorState<u8, Vec<u8>> {
        spectator::state_machine::SpectatorState::new(identity::GameId::from("game"))
    }

    fn interval() -> std::time::Duration {
        spectator::state_machine::MIN_LEASE_INTERVAL
    }

    fn spectating_state() -> spectator::state_machine::SpectatorState<u8, Vec<u8>> {
        let mut state = state();
        state.start_spectating(
            &identity::ZoneName::from("zone"),
            p2p::HistoryKeyPrefix::from("prefix"),
            time::UnixTime::from_secs(10),
            interval(),
        );
        state
    }

    #[test]
    fn test_usage() {
        let mut state = state();
        let outputs = handle_local_peer(
            &mut state,
            Some(identity::PeerId::from("me")),
            time::UnixTime::from_secs(10),
            interval(),
        );
        assert_eq!(state.me, Some(identity::PeerId::from("me")));
        assert_eq!(outputs, Vec::new());
    }

    #[test]
    fn learning_the_id_while_spectating_publishes_a_lease() {
        let mut state = spectating_state();
        let now = time::UnixTime::from_secs(20);
        let outputs = handle_local_peer(
            &mut state,
            Some(identity::PeerId::from("me")),
            now,
            interval(),
        );
        assert_eq!(
            outputs,
            vec![spectator::state_machine::SpectateOutput::PublishLease {
                zone: identity::ZoneName::from("zone"),
                spectator: identity::PeerId::from("me"),
                until: now.saturating_add(
                    spectator::state_machine::MIN_LEASE_INTERVAL
                        .saturating_mul(spectator::state_machine::LEASE_TTL_MULTIPLIER)
                ),
            }]
        );
    }

    #[test]
    fn repeating_the_same_id_publishes_nothing() {
        let mut state = spectating_state();
        handle_local_peer(
            &mut state,
            Some(identity::PeerId::from("me")),
            time::UnixTime::from_secs(20),
            interval(),
        );
        assert_eq!(
            handle_local_peer(
                &mut state,
                Some(identity::PeerId::from("me")),
                time::UnixTime::from_secs(30),
                interval(),
            ),
            Vec::new()
        );
    }
}
