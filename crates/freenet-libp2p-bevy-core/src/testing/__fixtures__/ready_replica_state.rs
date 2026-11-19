use crate::identity;
use crate::p2p;
use crate::replica_sync;
use crate::testing;

#[must_use]
pub fn ready_replica_state(
    zone: &identity::ZoneName,
) -> replica_sync::state_machine::ReplicaState<u8> {
    let mut state = replica_sync::state_machine::ReplicaState::<u8>::default();
    state.update(
        replica_sync::state_machine::Input::Joined {
            zone: zone.clone(),
            prefix: p2p::HistoryKeyPrefix::from("prefix"),
            members: Vec::new(),
            liveness: testing::alive_zone_liveness(),
        },
        replica_sync::ElapsedSinceAppStart::default(),
    );
    state.update(
        replica_sync::state_machine::Input::SnapshotMissingFromZoneMember,
        replica_sync::ElapsedSinceAppStart::default(),
    );
    state
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::replica_sync;

    use super::ready_replica_state;

    #[test]
    fn test_usage() {
        let state = ready_replica_state(&identity::ZoneName::from("r"));
        assert_eq!(state.phase, replica_sync::state_machine::Phase::Ready);
    }
}
