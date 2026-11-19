use crate::identity;
use crate::p2p;
use crate::spectator;
use crate::time;

pub fn start_spectating<Op, Snap>(
    state: &mut spectator::state_machine::SpectatorState<Op, Snap>,
    zone: &identity::ZoneName,
    history_prefix: p2p::HistoryKeyPrefix,
    now: time::UnixTime,
    lease_interval: std::time::Duration,
) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    let mut outputs = if state.zone.is_some() {
        state.stop_spectating(now)
    } else {
        Vec::new()
    };
    state.zone = Some(zone.clone());
    state.history_prefix = Some(history_prefix.clone());
    state.phase = spectator::state_machine::SpectatePhase::FetchingSnapshot;
    state.buffered_ops.clear();
    state.applied_metas.clear();
    state.last_slice_request_at.clear();
    state.pending_slice_requests.clear();
    outputs.push(spectator::state_machine::SpectateOutput::Subscribe(
        identity::Topic::from_replica_sync(&state.game, zone),
    ));
    outputs.push(spectator::state_machine::SpectateOutput::WatchZone(
        zone.clone(),
    ));
    outputs.push(spectator::state_machine::SpectateOutput::GetHistoryRecord(
        p2p::ZoneHistoryRecordKind::Manifest(history_prefix),
    ));
    outputs.extend(state.publish_lease(now, lease_interval));
    outputs
}

#[cfg(test)]
mod tests {
    use super::start_spectating;
    use crate::identity;
    use crate::p2p;
    use crate::spectator;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut state =
            testing::spectator_state(None, spectator::state_machine::SpectatePhase::Idle);
        state.me = Some(identity::PeerId::from("me"));
        let zone = identity::ZoneName::from("zone");
        let now = time::UnixTime::from_secs(100);
        let outputs = start_spectating(
            &mut state,
            &zone,
            p2p::HistoryKeyPrefix::from("prefix"),
            now,
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        assert!(matches!(
            outputs.as_slice(),
            [
                spectator::state_machine::SpectateOutput::Subscribe(replica_sync_topic),
                spectator::state_machine::SpectateOutput::WatchZone(watched),
                spectator::state_machine::SpectateOutput::GetHistoryRecord(
                    p2p::ZoneHistoryRecordKind::Manifest(_)
                ),
                spectator::state_machine::SpectateOutput::PublishLease { .. },
            ] if *replica_sync_topic == identity::Topic::from_replica_sync(&identity::GameId::from("game"), &zone)
                && *watched == zone
        ));
        assert_eq!(
            state.phase,
            spectator::state_machine::SpectatePhase::FetchingSnapshot
        );
        assert_eq!(state.zone, Some(zone));
        assert_eq!(
            state.history_prefix,
            Some(p2p::HistoryKeyPrefix::from("prefix"))
        );
    }

    #[test]
    fn starting_without_a_known_id_subscribes_but_publishes_no_lease() {
        let mut state =
            testing::spectator_state(None, spectator::state_machine::SpectatePhase::Idle);
        let zone = identity::ZoneName::from("zone");
        let outputs = start_spectating(
            &mut state,
            &zone,
            p2p::HistoryKeyPrefix::from("prefix"),
            time::UnixTime::from_secs(100),
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        assert!(matches!(
            outputs.as_slice(),
            [
                spectator::state_machine::SpectateOutput::Subscribe(_),
                spectator::state_machine::SpectateOutput::WatchZone(_),
                spectator::state_machine::SpectateOutput::GetHistoryRecord(_),
            ]
        ));
        assert_eq!(
            state.phase,
            spectator::state_machine::SpectatePhase::FetchingSnapshot
        );
    }

    #[test]
    fn starting_another_zone_leaves_the_previous_one_first() {
        let mut state =
            testing::spectator_state(None, spectator::state_machine::SpectatePhase::Idle);
        state.me = Some(identity::PeerId::from("me"));
        let first = identity::ZoneName::from("first");
        start_spectating(
            &mut state,
            &first,
            p2p::HistoryKeyPrefix::from("prefix"),
            time::UnixTime::from_secs(10),
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        let outputs = start_spectating(
            &mut state,
            &identity::ZoneName::from("second"),
            p2p::HistoryKeyPrefix::from("prefix"),
            time::UnixTime::from_secs(20),
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        let unsubscribe = spectator::state_machine::SpectateOutput::Unsubscribe(
            identity::Topic::from_replica_sync(&identity::GameId::from("game"), &first),
        );
        assert!(outputs.contains(&unsubscribe));
        assert_eq!(state.zone, Some(identity::ZoneName::from("second")));
        assert_eq!(
            state.phase,
            spectator::state_machine::SpectatePhase::FetchingSnapshot
        );
    }
}
