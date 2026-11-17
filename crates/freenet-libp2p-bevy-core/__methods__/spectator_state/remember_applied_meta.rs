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
        remember_applied_meta(&mut state, meta(1));
        assert_eq!(state.applied_metas, vec![meta(1)]);
    }

    #[test]
    fn the_oldest_meta_is_forgotten_beyond_the_window() {
        let mut state = spectator::state_machine::SpectatorState::<u8, Vec<u8>>::new(
            identity::GameId::from("game"),
            identity::PeerId::from("me"),
        );
        let cap = u32::try_from(spectator::state_machine::MAX_APPLIED_METAS).expect("cap");
        for lamport in 0..=cap {
            remember_applied_meta(&mut state, meta(lamport));
        }
        assert_eq!(
            state.applied_metas.len(),
            spectator::state_machine::MAX_APPLIED_METAS
        );
        assert_eq!(state.applied_metas.first(), Some(&meta(1)));
    }
}
