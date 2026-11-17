use crate::spectator;
use crate::time;

pub fn lease_until<Op, Snap>(
    state: &spectator::state_machine::SpectatorState<Op, Snap>,
    now: time::UnixTime,
) -> time::UnixTime {
    now.saturating_add(
        state
            .lease_interval()
            .saturating_mul(spectator::state_machine::LEASE_TTL_MULTIPLIER),
    )
}

#[cfg(test)]
mod tests {
    use super::lease_until;
    use crate::identity;
    use crate::spectator;
    use crate::time;

    #[test]
    fn test_usage() {
        let state = spectator::state_machine::SpectatorState::<u8, Vec<u8>>::new(
            identity::GameId::from("game"),
            identity::PeerId::from("me"),
        );
        assert_eq!(
            lease_until(&state, time::UnixTime::from_secs(10)),
            time::UnixTime::from_secs(55)
        );
    }
}
