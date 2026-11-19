use crate::identity;
use crate::replica_sync;
use crate::spectator;

pub fn handle_remote_op<Op, Snap>(
    state: &mut spectator::state_machine::SpectatorState<Op, Snap>,
    zone: &identity::ZoneName,
    meta: replica_sync::OpMeta,
    op: Op,
) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    if state.zone.as_ref() != Some(zone) || state.has_seen_meta(meta) {
        return Vec::new();
    }
    match state.phase {
        spectator::state_machine::SpectatePhase::Live => {
            state.remember_applied_meta(meta);
            vec![spectator::state_machine::SpectateOutput::Apply { meta, op }]
        }
        spectator::state_machine::SpectatePhase::FetchingSnapshot => {
            state.buffered_ops.push((meta, op));
            Vec::new()
        }
        spectator::state_machine::SpectatePhase::Idle => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::handle_remote_op;
    use crate::identity;
    use crate::spectator;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut state =
            testing::spectator_state(Some("zone"), spectator::state_machine::SpectatePhase::Idle);
        state.phase = spectator::state_machine::SpectatePhase::Live;
        let outputs = handle_remote_op(
            &mut state,
            &identity::ZoneName::from("zone"),
            testing::meta(1, 1),
            7_u8,
        );
        assert_eq!(
            outputs,
            vec![spectator::state_machine::SpectateOutput::Apply {
                meta: testing::meta(1, 1),
                op: 7_u8,
            }]
        );
    }

    #[test]
    fn ops_are_buffered_until_the_snapshot_arrives() {
        let mut state =
            testing::spectator_state(Some("zone"), spectator::state_machine::SpectatePhase::Idle);
        state.phase = spectator::state_machine::SpectatePhase::FetchingSnapshot;
        let zone = identity::ZoneName::from("zone");
        assert_eq!(
            handle_remote_op(&mut state, &zone, testing::meta(2, 1), 2_u8),
            Vec::new()
        );
        assert_eq!(state.buffered_ops.len(), 1);
        assert_eq!(state.applied_metas, Vec::new());
    }

    #[test]
    fn a_seen_op_is_never_applied_twice() {
        let mut state =
            testing::spectator_state(Some("zone"), spectator::state_machine::SpectatePhase::Idle);
        state.phase = spectator::state_machine::SpectatePhase::Live;
        let zone = identity::ZoneName::from("zone");
        handle_remote_op(&mut state, &zone, testing::meta(1, 1), 2_u8);
        assert_eq!(
            handle_remote_op(&mut state, &zone, testing::meta(1, 1), 2_u8),
            Vec::new()
        );
        assert_eq!(state.applied_metas, vec![testing::meta(1, 1)]);
    }

    #[test]
    fn an_op_for_another_zone_is_ignored() {
        let mut state =
            testing::spectator_state(Some("zone"), spectator::state_machine::SpectatePhase::Idle);
        state.phase = spectator::state_machine::SpectatePhase::Live;
        assert_eq!(
            handle_remote_op(
                &mut state,
                &identity::ZoneName::from("other"),
                testing::meta(1, 1),
                2_u8,
            ),
            Vec::new()
        );
    }
}
