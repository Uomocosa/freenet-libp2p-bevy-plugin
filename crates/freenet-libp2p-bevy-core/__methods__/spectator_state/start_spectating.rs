use crate::identity;
use crate::p2p;
use crate::spectator;
use crate::time;

pub fn start_spectating<Op, Snap>(
    state: &mut spectator::state_machine::SpectatorState<Op, Snap>,
    zone: &identity::ZoneName,
    history_prefix: p2p::HistoryKeyPrefix,
    now: time::UnixTime,
) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    let mut outputs = if state.zone.is_some() {
        state.stop_spectating(now)
    } else {
        Vec::new()
    };
    state.zone = Some(zone.clone());
    state.phase = spectator::state_machine::SpectatePhase::FetchingSnapshot;
    state.buffered_ops.clear();
    state.applied_metas.clear();
    state.last_slice_request_at.clear();
    state.pending_slice_requests.clear();
    outputs.push(spectator::state_machine::SpectateOutput::Subscribe(
        identity::Topic::from_replica_sync(&state.game, zone),
    ));
    outputs.push(spectator::state_machine::SpectateOutput::GetHistoryRecord(
        p2p::ZoneHistoryRecordKind::Manifest(history_prefix),
    ));
    outputs.extend(state.publish_lease(now));
    outputs
}

#[cfg(test)]
mod tests {
    use super::start_spectating;
    use crate::identity;
    use crate::p2p;
    use crate::spectator;
    use crate::time;

    fn state() -> spectator::state_machine::SpectatorState<u8, Vec<u8>> {
        spectator::state_machine::SpectatorState::new(
            identity::GameId::from("game"),
            identity::PeerId::from("me"),
        )
    }

    fn prefix() -> p2p::HistoryKeyPrefix {
        p2p::HistoryKeyPrefix::from("prefix")
    }

    #[test]
    fn test_usage() {
        let mut state = state();
        let zone = identity::ZoneName::from("zone");
        let now = time::UnixTime::from_secs(100);
        let outputs = start_spectating(&mut state, &zone, prefix(), now);
        assert!(matches!(
            outputs.as_slice(),
            [
                spectator::state_machine::SpectateOutput::Subscribe(topic),
                spectator::state_machine::SpectateOutput::GetHistoryRecord(
                    p2p::ZoneHistoryRecordKind::Manifest(_)
                ),
                spectator::state_machine::SpectateOutput::PublishLease { .. },
            ] if *topic == identity::Topic::from_replica_sync(&identity::GameId::from("game"), &zone)
        ));
        assert_eq!(
            state.phase,
            spectator::state_machine::SpectatePhase::FetchingSnapshot
        );
        assert_eq!(state.zone, Some(zone));
    }

    #[test]
    fn starting_another_zone_leaves_the_previous_one_first() {
        let mut state = state();
        let first = identity::ZoneName::from("first");
        start_spectating(&mut state, &first, prefix(), time::UnixTime::from_secs(10));
        let outputs = start_spectating(
            &mut state,
            &identity::ZoneName::from("second"),
            prefix(),
            time::UnixTime::from_secs(20),
        );
        let unsubscribe = spectator::state_machine::SpectateOutput::Unsubscribe(
            identity::Topic::from_replica_sync(&identity::GameId::from("game"), &first),
        );
        assert!(outputs.contains(&unsubscribe));
        assert!(outputs.contains(&spectator::state_machine::SpectateOutput::Clear));
        assert_eq!(state.zone, Some(identity::ZoneName::from("second")));
        assert_eq!(
            state.phase,
            spectator::state_machine::SpectatePhase::FetchingSnapshot
        );
    }
}
