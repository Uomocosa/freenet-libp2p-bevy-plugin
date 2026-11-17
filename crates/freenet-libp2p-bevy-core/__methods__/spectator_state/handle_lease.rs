use crate::identity;
use crate::spectator;
use crate::time;

pub fn handle_lease<Op, Snap>(
    state: &mut spectator::state_machine::SpectatorState<Op, Snap>,
    zone: &identity::ZoneName,
    spectator: &identity::PeerId,
    until: time::UnixTime,
    now: time::UnixTime,
) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    if state.zone.as_ref().is_some_and(|current| current != zone) {
        return Vec::new();
    }
    if until <= now {
        state.leases.remove(spectator);
        return state.expire_leases(now);
    }
    state.leases.insert(spectator.clone(), until);
    state.expire_leases(now)
}

#[cfg(test)]
mod tests {
    use super::handle_lease;
    use crate::identity;
    use crate::spectator;
    use crate::time;

    fn state() -> spectator::state_machine::SpectatorState<u8, Vec<u8>> {
        spectator::state_machine::SpectatorState::new(
            identity::GameId::from("game"),
            identity::PeerId::from("me"),
        )
    }

    #[test]
    fn test_usage() {
        let mut state = state();
        let zone = identity::ZoneName::from("zone");
        let outputs = handle_lease(
            &mut state,
            &zone,
            &identity::PeerId::from("a"),
            time::UnixTime::from_secs(10),
            time::UnixTime::from_secs(1),
        );
        assert_eq!(
            outputs,
            vec![spectator::state_machine::SpectateOutput::SpectatorCountChanged(1)]
        );
        assert_eq!(state.leases.len(), 1);
    }

    #[test]
    fn an_expired_lease_is_ignored() {
        let mut state = state();
        let outputs = handle_lease(
            &mut state,
            &identity::ZoneName::from("zone"),
            &identity::PeerId::from("a"),
            time::UnixTime::from_secs(1),
            time::UnixTime::from_secs(1),
        );
        assert_eq!(outputs, Vec::new());
        assert!(state.leases.is_empty());
    }

    #[test]
    fn a_lease_for_another_zone_is_ignored_while_spectating() {
        let mut state = state();
        state.zone = Some(identity::ZoneName::from("zone"));
        let outputs = handle_lease(
            &mut state,
            &identity::ZoneName::from("other"),
            &identity::PeerId::from("a"),
            time::UnixTime::from_secs(10),
            time::UnixTime::from_secs(1),
        );
        assert_eq!(outputs, Vec::new());
        assert!(state.leases.is_empty());
    }

    #[test]
    fn an_expired_lease_for_a_known_spectator_removes_it() {
        let mut state = state();
        let zone = identity::ZoneName::from("zone");
        let spectator = identity::PeerId::from("a");
        handle_lease(
            &mut state,
            &zone,
            &spectator,
            time::UnixTime::from_secs(10),
            time::UnixTime::from_secs(1),
        );
        let outputs = handle_lease(
            &mut state,
            &zone,
            &spectator,
            time::UnixTime::from_secs(2),
            time::UnixTime::from_secs(2),
        );
        assert_eq!(
            outputs,
            vec![spectator::state_machine::SpectateOutput::SpectatorCountChanged(0)]
        );
        assert!(state.leases.is_empty());
    }
}
