use crate::identity;
use crate::spectator;

pub fn handle_my_zone<Op, Snap>(
    state: &mut spectator::state_machine::SpectatorState<Op, Snap>,
    zone: Option<identity::ZoneName>,
) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    if state.my_zone == zone {
        return Vec::new();
    }
    state.my_zone = zone;
    state.leases.clear();
    if state.last_reported_count == 0 {
        return Vec::new();
    }
    state.last_reported_count = 0;
    vec![spectator::state_machine::SpectateOutput::SpectatorCountChanged(0)]
}

#[cfg(test)]
mod tests {
    use super::handle_my_zone;
    use crate::identity;
    use crate::spectator;
    use crate::time;

    fn state() -> spectator::state_machine::SpectatorState<u8, Vec<u8>> {
        let mut state =
            spectator::state_machine::SpectatorState::new(identity::GameId::from("game"));
        state.my_zone = Some(identity::ZoneName::from("home"));
        state
            .leases
            .insert(identity::PeerId::from("a"), time::UnixTime::from_secs(10));
        state.last_reported_count = 1;
        state
    }

    #[test]
    fn test_usage() {
        let mut state = state();
        let outputs = handle_my_zone(&mut state, Some(identity::ZoneName::from("new")));
        assert_eq!(
            outputs,
            vec![spectator::state_machine::SpectateOutput::SpectatorCountChanged(0)]
        );
        assert!(state.leases.is_empty());
        assert_eq!(state.my_zone, Some(identity::ZoneName::from("new")));
    }

    #[test]
    fn the_same_zone_changes_nothing() {
        let mut state = state();
        assert_eq!(
            handle_my_zone(&mut state, Some(identity::ZoneName::from("home"))),
            Vec::new()
        );
        assert_eq!(state.leases.len(), 1);
        assert_eq!(state.last_reported_count, 1);
    }

    #[test]
    fn leaving_my_zone_resets_a_zero_count_silently() {
        let mut state = state();
        state.leases.clear();
        state.last_reported_count = 0;
        assert_eq!(handle_my_zone(&mut state, None), Vec::new());
        assert_eq!(state.my_zone, None);
    }
}
