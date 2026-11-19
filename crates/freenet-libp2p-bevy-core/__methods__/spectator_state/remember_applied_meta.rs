use crate::replica_sync;
use crate::spectator;

pub fn remember_applied_meta<Op, Snap>(
    state: &mut spectator::state_machine::SpectatorState<Op, Snap>,
    meta: replica_sync::OpMeta,
) {
    state.applied_metas.push(meta);
    if state.applied_metas.len() > spectator::state_machine::MAX_APPLIED_METAS {
        state.applied_metas.remove(0);
    }
}

#[cfg(test)]
mod tests {
    use super::remember_applied_meta;
    use crate::spectator;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut state =
            testing::spectator_state(None, spectator::state_machine::SpectatePhase::Idle);
        remember_applied_meta(&mut state, testing::meta(1, 1));
        assert_eq!(state.applied_metas, vec![testing::meta(1, 1)]);
    }

    #[test]
    fn the_oldest_meta_is_forgotten_beyond_the_window() {
        let mut state =
            testing::spectator_state(None, spectator::state_machine::SpectatePhase::Idle);
        let cap = u32::try_from(spectator::state_machine::MAX_APPLIED_METAS).expect("cap");
        for lamport in 0..=cap {
            remember_applied_meta(&mut state, testing::meta(lamport, 1));
        }
        assert_eq!(
            state.applied_metas.len(),
            spectator::state_machine::MAX_APPLIED_METAS
        );
        assert_eq!(state.applied_metas.first(), Some(&testing::meta(1, 1)));
    }
}
