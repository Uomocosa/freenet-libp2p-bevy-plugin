use crate::identity;
use crate::replica_sync;
use crate::spectator;
use crate::time;

pub fn handle_remote_slice_content_hashes<Op, Snap>(
    state: &mut spectator::state_machine::SpectatorState<Op, Snap>,
    from: &identity::PeerId,
    zone: &identity::ZoneName,
    local: &replica_sync::ReplicaSliceContentHashes,
    remote: &replica_sync::ReplicaSliceContentHashes,
    now: time::UnixTime,
) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    if state.phase != spectator::state_machine::SpectatePhase::Live
        || state.zone.as_ref() != Some(zone)
        || state.players.contains(from)
    {
        return Vec::new();
    }
    if state.last_slice_request_at.get(from).is_some_and(|last| {
        now.elapsed_since(*last) < spectator::state_machine::SLICE_REQUEST_COOLDOWN
    }) {
        return Vec::new();
    }
    let requests: Vec<spectator::state_machine::SpectateOutput<Op, Snap>> =
        replica_sync::ReplicaSliceIndex::from_hash_mismatches(local, remote)
            .into_iter()
            .take(spectator::state_machine::MAX_SLICE_REQUESTS_PER_PEER)
            .map(
                |index| spectator::state_machine::SpectateOutput::RequestSlice {
                    from: from.clone(),
                    index,
                },
            )
            .collect();
    if requests.is_empty() {
        return Vec::new();
    }
    state.last_slice_request_at.insert(from.clone(), now);
    for request in &requests {
        if let spectator::state_machine::SpectateOutput::RequestSlice { index, .. } = request {
            state.pending_slice_requests.insert(*index, now);
        }
    }
    requests
}

#[cfg(test)]
mod tests {
    use super::handle_remote_slice_content_hashes;
    use crate::identity;
    use crate::replica_sync;
    use crate::spectator;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut state =
            testing::spectator_state(Some("zone"), spectator::state_machine::SpectatePhase::Live);
        let outputs = handle_remote_slice_content_hashes(
            &mut state,
            &identity::PeerId::from("holder"),
            &identity::ZoneName::from("zone"),
            &testing::hashes(&[(0, 1)]),
            &testing::hashes(&[(0, 1), (1, 2)]),
            time::UnixTime::from_secs(1),
        );
        assert_eq!(testing::requested_replica_slice_indices(&outputs), vec![1]);
        assert_eq!(
            outputs[0],
            spectator::state_machine::SpectateOutput::RequestSlice {
                from: identity::PeerId::from("holder"),
                index: replica_sync::ReplicaSliceIndex(1),
            }
        );
        assert_eq!(
            state
                .pending_slice_requests
                .get(&replica_sync::ReplicaSliceIndex(1)),
            Some(&time::UnixTime::from_secs(1))
        );
    }

    #[test]
    fn a_slice_is_never_requested_from_a_zone_player() {
        let mut state =
            testing::spectator_state(Some("zone"), spectator::state_machine::SpectatePhase::Live);
        state.players.insert(identity::PeerId::from("player"));
        let outputs = handle_remote_slice_content_hashes(
            &mut state,
            &identity::PeerId::from("player"),
            &identity::ZoneName::from("zone"),
            &testing::hashes(&[(0, 1)]),
            &testing::hashes(&[(0, 9)]),
            time::UnixTime::from_secs(1),
        );
        assert_eq!(outputs, Vec::new());
    }

    #[test]
    fn requests_to_one_peer_are_rate_limited() {
        let mut state =
            testing::spectator_state(Some("zone"), spectator::state_machine::SpectatePhase::Live);
        let peer = identity::PeerId::from("holder");
        let zone = identity::ZoneName::from("zone");
        let local = testing::hashes(&[(0, 1)]);
        let remote = testing::hashes(&[(0, 9)]);
        handle_remote_slice_content_hashes(
            &mut state,
            &peer,
            &zone,
            &local,
            &remote,
            time::UnixTime::from_secs(1),
        );
        let repeated = handle_remote_slice_content_hashes(
            &mut state,
            &peer,
            &zone,
            &local,
            &remote,
            time::UnixTime::from_secs(2),
        );
        assert_eq!(repeated, Vec::new());
        let later = handle_remote_slice_content_hashes(
            &mut state,
            &peer,
            &zone,
            &local,
            &remote,
            time::UnixTime::from_secs(10),
        );
        assert_eq!(testing::requested_replica_slice_indices(&later), vec![0]);
    }

    #[test]
    fn a_compare_before_the_snapshot_requests_nothing() {
        let mut state =
            testing::spectator_state(Some("zone"), spectator::state_machine::SpectatePhase::Live);
        state.phase = spectator::state_machine::SpectatePhase::FetchingSnapshot;
        let outputs = handle_remote_slice_content_hashes(
            &mut state,
            &identity::PeerId::from("holder"),
            &identity::ZoneName::from("zone"),
            &testing::hashes(&[]),
            &testing::hashes(&[(0, 9)]),
            time::UnixTime::from_secs(1),
        );
        assert_eq!(outputs, Vec::new());
    }

    #[test]
    fn one_compare_requests_at_most_the_per_peer_cap() {
        let mut state =
            testing::spectator_state(Some("zone"), spectator::state_machine::SpectatePhase::Live);
        let remote = testing::hashes(&[(0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1)]);
        let outputs = handle_remote_slice_content_hashes(
            &mut state,
            &identity::PeerId::from("holder"),
            &identity::ZoneName::from("zone"),
            &testing::hashes(&[]),
            &remote,
            time::UnixTime::from_secs(1),
        );
        assert_eq!(
            testing::requested_replica_slice_indices(&outputs).len(),
            spectator::state_machine::MAX_SLICE_REQUESTS_PER_PEER
        );
    }
}
