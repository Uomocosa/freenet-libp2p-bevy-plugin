use crate::identity;
use crate::spectator;

#[must_use]
pub fn spectator_state(
    zone: Option<&str>,
    phase: spectator::state_machine::SpectatePhase,
) -> spectator::state_machine::SpectatorState<u8, Vec<u8>> {
    let mut state = spectator::state_machine::SpectatorState::new(identity::GameId::from("game"));
    state.zone = zone.map(identity::ZoneName::from);
    state.phase = phase;
    state
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::spectator;

    use super::spectator_state;

    #[test]
    fn test_usage() {
        let state = spectator_state(Some("zone"), spectator::state_machine::SpectatePhase::Live);
        assert_eq!(state.game, identity::GameId::from("game"));
        assert_eq!(state.zone, Some(identity::ZoneName::from("zone")));
        assert_eq!(state.phase, spectator::state_machine::SpectatePhase::Live);
        assert_eq!(state.me, None);
    }
}
