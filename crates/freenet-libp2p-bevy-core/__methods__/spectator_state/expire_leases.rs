use crate::spectator;
use crate::time;

pub fn expire_leases<Op, Snap>(
    state: &mut spectator::state_machine::SpectatorState<Op, Snap>,
    now: time::UnixTime,
) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    state.leases.retain(|_, until| *until > now);
    let count = state.leases.len();
    if count == state.last_reported_count {
        return Vec::new();
    }
    state.last_reported_count = count;
    vec![spectator::state_machine::SpectateOutput::SpectatorCountChanged(count)]
}

#[cfg(test)]
mod tests {
    use super::expire_leases;
    use crate::identity;
    use crate::spectator;
    use crate::time;

    fn state() -> spectator::state_machine::SpectatorState<u8, Vec<u8>> {
        spectator::state_machine::SpectatorState::new(identity::GameId::from("game"))
    }

    #[test]
    fn test_usage() {
        let mut state = state();
        state
            .leases
            .insert(identity::PeerId::from("a"), time::UnixTime::from_secs(10));
        let added = expire_leases(&mut state, time::UnixTime::from_secs(5));
        assert_eq!(
            added,
            vec![spectator::state_machine::SpectateOutput::SpectatorCountChanged(1)]
        );
        let expired = expire_leases(&mut state, time::UnixTime::from_secs(11));
        assert_eq!(
            expired,
            vec![spectator::state_machine::SpectateOutput::SpectatorCountChanged(0)]
        );
        assert!(state.leases.is_empty());
    }

    #[test]
    fn an_unchanged_count_is_not_reported_again() {
        let mut state = state();
        state
            .leases
            .insert(identity::PeerId::from("a"), time::UnixTime::from_secs(10));
        expire_leases(&mut state, time::UnixTime::from_secs(5));
        assert_eq!(
            expire_leases(&mut state, time::UnixTime::from_secs(6)),
            Vec::new()
        );
        assert_eq!(state.last_reported_count, 1);
    }
}
