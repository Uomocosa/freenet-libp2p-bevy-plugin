use crate::identity;
use crate::spectator;
use crate::time;

pub fn stop_spectating<Op, Snap>(
    state: &mut spectator::state_machine::SpectatorState<Op, Snap>,
    now: time::UnixTime,
) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    let Some(zone) = state.zone.take() else {
        return Vec::new();
    };
    let mut outputs = Vec::new();
    if let Some(me) = state.me.clone() {
        outputs.push(spectator::state_machine::SpectateOutput::PublishLease {
            zone: zone.clone(),
            spectator: me,
            until: now,
        });
    }
    state.phase = spectator::state_machine::SpectatePhase::Idle;
    state.history_prefix = None;
    state.buffered_ops.clear();
    state.applied_metas.clear();
    state.last_slice_request_at.clear();
    state.pending_slice_requests.clear();
    state.next_lease_at = time::UnixTime::EPOCH;
    outputs.push(spectator::state_machine::SpectateOutput::Unsubscribe(
        identity::Topic::from_replica_sync(&state.game, &zone),
    ));
    outputs.push(spectator::state_machine::SpectateOutput::UnwatchZone(zone));
    outputs
}

#[cfg(test)]
mod tests {
    use super::stop_spectating;
    use crate::identity;
    use crate::spectator;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut state =
            testing::spectator_state(Some("zone"), spectator::state_machine::SpectatePhase::Live);
        state.me = Some(identity::PeerId::from("me"));
        state.buffered_ops.push((testing::meta(1, 1), 2_u8));
        let now = time::UnixTime::from_secs(50);
        let outputs = stop_spectating(&mut state, now);
        assert_eq!(
            outputs,
            vec![
                spectator::state_machine::SpectateOutput::PublishLease {
                    zone: identity::ZoneName::from("zone"),
                    spectator: identity::PeerId::from("me"),
                    until: now,
                },
                spectator::state_machine::SpectateOutput::Unsubscribe(
                    identity::Topic::from_replica_sync(
                        &identity::GameId::from("game"),
                        &identity::ZoneName::from("zone"),
                    )
                ),
                spectator::state_machine::SpectateOutput::UnwatchZone(identity::ZoneName::from(
                    "zone"
                )),
            ]
        );
        assert_eq!(state.phase, spectator::state_machine::SpectatePhase::Idle);
        assert_eq!(state.zone, None);
        assert_eq!(state.buffered_ops, Vec::new());
    }

    #[test]
    fn stopping_while_idle_does_nothing() {
        let mut state =
            testing::spectator_state(None, spectator::state_machine::SpectatePhase::Idle);
        assert_eq!(
            stop_spectating(&mut state, time::UnixTime::from_secs(1)),
            Vec::new()
        );
    }

    #[test]
    fn stopping_without_a_known_id_skips_the_lease_but_still_unwatches() {
        let mut state =
            testing::spectator_state(Some("zone"), spectator::state_machine::SpectatePhase::Live);
        let outputs = stop_spectating(&mut state, time::UnixTime::from_secs(50));
        assert_eq!(
            outputs,
            vec![
                spectator::state_machine::SpectateOutput::Unsubscribe(
                    identity::Topic::from_replica_sync(
                        &identity::GameId::from("game"),
                        &identity::ZoneName::from("zone"),
                    )
                ),
                spectator::state_machine::SpectateOutput::UnwatchZone(identity::ZoneName::from(
                    "zone"
                )),
            ]
        );
        assert_eq!(state.zone, None);
    }
}
