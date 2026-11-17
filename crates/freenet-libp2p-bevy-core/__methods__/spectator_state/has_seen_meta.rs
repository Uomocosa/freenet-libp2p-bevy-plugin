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
    use crate::identity;
    use crate::replica_sync;
    use crate::spectator;

    fn meta(lamport: u32) -> replica_sync::OpMeta {
        replica_sync::OpMeta { lamport, author: 1 }
    }

    #[test]
    fn test_usage() {
        let mut state = spectator::state_machine::SpectatorState::<u8, Vec<u8>>::new(
            identity::GameId::from("game"),
            identity::PeerId::from("me"),
        );
        assert!(!has_seen_meta(&state, meta(1)));
        state.applied_metas.push(meta(1));
        assert!(has_seen_meta(&state, meta(1)));
    }

    #[test]
    fn a_buffered_op_counts_as_seen() {
        let mut state = spectator::state_machine::SpectatorState::<u8, Vec<u8>>::new(
            identity::GameId::from("game"),
            identity::PeerId::from("me"),
        );
        state.buffered_ops.push((meta(3), 4_u8));
        assert!(has_seen_meta(&state, meta(3)));
    }
}
