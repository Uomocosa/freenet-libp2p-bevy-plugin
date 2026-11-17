use crate::spectator;
use crate::time;

pub fn publish_lease<Op, Snap>(
    state: &mut spectator::state_machine::SpectatorState<Op, Snap>,
    now: time::UnixTime,
) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    let Some(zone) = state.zone.clone() else {
        return Vec::new();
    };
    let until = state.lease_until(now);
    state.next_lease_at = state.next_lease_after(now);
    vec![spectator::state_machine::SpectateOutput::PublishLease {
        zone,
        spectator: state.me.clone(),
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
        let mut state = spectator::state_machine::SpectatorState::new(
            identity::GameId::from("game"),
            identity::PeerId::from("me"),
        );
        state.zone = Some(identity::ZoneName::from("zone"));
        state
    }

    #[test]
    fn test_usage() {
        let mut state = state();
        let now = time::UnixTime::from_secs(100);
        let outputs = publish_lease(&mut state, now);
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
            identity::PeerId::from("me"),
        );
        assert_eq!(
            publish_lease(&mut state, time::UnixTime::from_secs(1)),
            Vec::new()
        );
    }
}
