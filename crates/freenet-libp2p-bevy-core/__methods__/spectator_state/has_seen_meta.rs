use crate::replica_sync;
use crate::spectator;

pub fn has_seen_meta<Op, Snap>(
    state: &spectator::state_machine::SpectatorState<Op, Snap>,
    meta: replica_sync::OpMeta,
) -> bool {
    state.applied_metas.contains(&meta)
        || state
            .buffered_ops
            .iter()
            .any(|(buffered, _)| *buffered == meta)
}

#[cfg(test)]
mod tests {
    use super::has_seen_meta;
    use crate::spectator;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut state =
            testing::spectator_state(None, spectator::state_machine::SpectatePhase::Idle);
        assert!(!has_seen_meta(&state, testing::meta(1, 1)));
        state.applied_metas.push(testing::meta(1, 1));
        assert!(has_seen_meta(&state, testing::meta(1, 1)));
    }

    #[test]
    fn a_buffered_op_counts_as_seen() {
        let mut state =
            testing::spectator_state(None, spectator::state_machine::SpectatePhase::Idle);
        state.buffered_ops.push((testing::meta(3, 1), 4_u8));
        assert!(has_seen_meta(&state, testing::meta(3, 1)));
    }
}
