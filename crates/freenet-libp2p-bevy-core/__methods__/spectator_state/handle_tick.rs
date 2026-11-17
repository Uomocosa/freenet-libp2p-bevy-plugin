use crate::spectator;
use crate::time;

pub fn handle_tick<Op, Snap>(
    state: &mut spectator::state_machine::SpectatorState<Op, Snap>,
    now: time::UnixTime,
) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    let mut outputs = state.expire_leases(now);
    outputs.extend(state.expire_slice_requests(now));
    if state.zone.is_some() && now >= state.next_lease_at {
        outputs.extend(state.publish_lease(now));
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
            identity::PeerId::from("me"),
        );
        assert_eq!(handle_tick(&mut state, time::UnixTime::EPOCH), Vec::new());
    }
}
