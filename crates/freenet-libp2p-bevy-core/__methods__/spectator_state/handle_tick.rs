use crate::spectator;
use crate::time;

pub fn handle_tick<Op, Snap>(
    state: &mut spectator::state_machine::SpectatorState<Op, Snap>,
    now: time::UnixTime,
    lease_interval: std::time::Duration,
) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    let mut outputs = state.expire_slice_requests(now);
    if state.zone.is_some() && now >= state.next_lease_at {
        outputs.extend(state.publish_lease(now, lease_interval));
    }
    outputs
}

#[cfg(test)]
mod tests {
    use super::handle_tick;
    use crate::identity;
    use crate::spectator;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut state = spectator::state_machine::SpectatorState::<u8, Vec<u8>>::new(
            identity::GameId::from("game"),
        );
        assert_eq!(
            handle_tick(
                &mut state,
                time::UnixTime::EPOCH,
                spectator::state_machine::MIN_LEASE_INTERVAL
            ),
            Vec::new()
        );
    }
}
