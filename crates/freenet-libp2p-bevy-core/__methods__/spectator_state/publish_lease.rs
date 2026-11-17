use crate::spectator;
use crate::time;

pub fn publish_lease<Op, Snap>(
    state: &mut spectator::state_machine::SpectatorState<Op, Snap>,
    now: time::UnixTime,
    lease_interval: std::time::Duration,
) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    let Some(zone) = state.zone.clone() else {
        return Vec::new();
    };
    let Some(me) = state.me.clone() else {
        return Vec::new();
    };
    let until = now.saturating_add(
        lease_interval.saturating_mul(spectator::state_machine::LEASE_TTL_MULTIPLIER),
    );
    state.next_lease_at = now.saturating_add(lease_interval.checked_div(2).unwrap_or_default());
    vec![spectator::state_machine::SpectateOutput::PublishLease {
        zone,
        spectator: me,
        until,
    }]
}

#[cfg(test)]
mod tests {
    use super::publish_lease;
    use crate::identity;
    use crate::spectator;
    use crate::time;

    fn state() -> spectator::state_machine::SpectatorState<u8, Vec<u8>> {
        let mut state =
            spectator::state_machine::SpectatorState::new(identity::GameId::from("game"));
        state.me = Some(identity::PeerId::from("me"));
        state.zone = Some(identity::ZoneName::from("zone"));
        state
    }

    #[test]
    fn test_usage() {
        let mut state = state();
        let now = time::UnixTime::from_secs(100);
        let outputs = publish_lease(
            &mut state,
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
        assert_eq!(
            state.next_lease_at,
            now.saturating_add(
                spectator::state_machine::MIN_LEASE_INTERVAL
                    .checked_div(2)
                    .unwrap_or_default()
            )
        );
    }

    #[test]
    fn an_idle_state_publishes_no_lease() {
        let mut state = spectator::state_machine::SpectatorState::<u8, Vec<u8>>::new(
            identity::GameId::from("game"),
        );
        state.me = Some(identity::PeerId::from("me"));
        assert_eq!(
            publish_lease(
                &mut state,
                time::UnixTime::from_secs(1),
                spectator::state_machine::MIN_LEASE_INTERVAL
            ),
            Vec::new()
        );
    }

    #[test]
    fn no_lease_is_published_before_the_local_id_is_known() {
        let mut state = spectator::state_machine::SpectatorState::<u8, Vec<u8>>::new(
            identity::GameId::from("game"),
        );
        state.zone = Some(identity::ZoneName::from("zone"));
        assert_eq!(
            publish_lease(
                &mut state,
                time::UnixTime::from_secs(1),
                spectator::state_machine::MIN_LEASE_INTERVAL
            ),
            Vec::new()
        );
        assert_eq!(state.next_lease_at, time::UnixTime::EPOCH);
    }
}
