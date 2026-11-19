use crate::spectator;

pub fn handle_snapshot<Op, Snap>(
    state: &mut spectator::state_machine::SpectatorState<Op, Snap>,
    snapshot: Snap,
) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    if state.phase != spectator::state_machine::SpectatePhase::FetchingSnapshot {
        return Vec::new();
    }
    state.phase = spectator::state_machine::SpectatePhase::Live;
    state
        .buffered_ops
        .sort_by_key(|(meta, _)| (meta.lamport, meta.author));
    let buffered = std::mem::take(&mut state.buffered_ops);
    let mut outputs = vec![spectator::state_machine::SpectateOutput::Merge(snapshot)];
    for (meta, op) in buffered {
        state.remember_applied_meta(meta);
        outputs.push(spectator::state_machine::SpectateOutput::Apply { meta, op });
    }
    outputs
}

#[cfg(test)]
mod tests {
    use super::handle_snapshot;
    use crate::identity;
    use crate::replica_sync;
    use crate::spectator;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut state = testing::spectator_state(
            Some("zone"),
            spectator::state_machine::SpectatePhase::FetchingSnapshot,
        );
        let outputs = handle_snapshot(&mut state, vec![1, 2, 3]);
        assert_eq!(
            outputs,
            vec![spectator::state_machine::SpectateOutput::Merge(vec![
                1, 2, 3
            ])]
        );
        assert_eq!(state.phase, spectator::state_machine::SpectatePhase::Live);
    }

    #[test]
    fn buffered_ops_are_applied_in_meta_order_after_the_merge() {
        let mut state = testing::spectator_state(
            Some("zone"),
            spectator::state_machine::SpectatePhase::FetchingSnapshot,
        );
        state.buffered_ops.push((testing::meta(3, 1), 3_u8));
        state.buffered_ops.push((testing::meta(1, 2), 1_u8));
        state.buffered_ops.push((testing::meta(2, 1), 2_u8));
        let outputs = handle_snapshot(&mut state, vec![0]);
        assert_eq!(
            outputs,
            vec![
                spectator::state_machine::SpectateOutput::Merge(vec![0]),
                spectator::state_machine::SpectateOutput::Apply {
                    meta: testing::meta(1, 2),
                    op: 1_u8,
                },
                spectator::state_machine::SpectateOutput::Apply {
                    meta: testing::meta(2, 1),
                    op: 2_u8,
                },
                spectator::state_machine::SpectateOutput::Apply {
                    meta: testing::meta(3, 1),
                    op: 3_u8,
                },
            ]
        );
        assert_eq!(state.buffered_ops, Vec::new());
        assert_eq!(state.applied_metas.len(), 3);
    }

    #[test]
    fn a_buffered_op_is_not_replayed_live_after_the_merge() {
        let mut state = testing::spectator_state(
            Some("zone"),
            spectator::state_machine::SpectatePhase::FetchingSnapshot,
        );
        let zone = identity::ZoneName::from("zone");
        state.update(
            spectator::state_machine::SpectateInput::Topic(
                replica_sync::ReplicaSyncTopicMessage::ApplyThisOp {
                    zone: zone.clone(),
                    meta: testing::meta(1, 1),
                    op: 9_u8,
                },
            ),
            time::UnixTime::EPOCH,
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        handle_snapshot(&mut state, vec![0]);
        let repeated = state.update(
            spectator::state_machine::SpectateInput::Topic(
                replica_sync::ReplicaSyncTopicMessage::ApplyThisOp {
                    zone,
                    meta: testing::meta(1, 1),
                    op: 9_u8,
                },
            ),
            time::UnixTime::EPOCH,
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        assert_eq!(repeated, Vec::new());
    }

    #[test]
    fn a_snapshot_in_the_live_phase_is_ignored() {
        let mut state = testing::spectator_state(
            Some("zone"),
            spectator::state_machine::SpectatePhase::FetchingSnapshot,
        );
        state.phase = spectator::state_machine::SpectatePhase::Live;
        assert_eq!(handle_snapshot(&mut state, vec![1]), Vec::new());
    }
}
