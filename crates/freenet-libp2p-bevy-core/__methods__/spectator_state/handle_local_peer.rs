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
    use crate::spectator;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut state =
            testing::spectator_state(None, spectator::state_machine::SpectatePhase::Idle);
        let outputs = handle_local_peer(
            &mut state,
            Some(identity::PeerId::from("me")),
            time::UnixTime::from_secs(10),
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        assert_eq!(state.me, Some(identity::PeerId::from("me")));
        assert_eq!(outputs, Vec::new());
    }

    #[test]
    fn learning_the_id_while_spectating_publishes_a_lease() {
        let mut state = testing::spectating_state(
            None,
            time::UnixTime::from_secs(10),
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        let now = time::UnixTime::from_secs(20);
        let outputs = handle_local_peer(
            &mut state,
            Some(identity::PeerId::from("me")),
            now,
            spectator::state_machine::MIN_LEASE_INTERVAL,
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
        let mut state = testing::spectating_state(
            None,
            time::UnixTime::from_secs(10),
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        handle_local_peer(
            &mut state,
            Some(identity::PeerId::from("me")),
            time::UnixTime::from_secs(20),
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        assert_eq!(
            handle_local_peer(
                &mut state,
                Some(identity::PeerId::from("me")),
                time::UnixTime::from_secs(30),
                spectator::state_machine::MIN_LEASE_INTERVAL,
            ),
            Vec::new()
        );
    }
}
