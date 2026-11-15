use crate::replica_sync;

pub fn expire_overdue_snapshot_part_request<Op>(
    state: &mut replica_sync::state_machine::ReplicaState<Op>,
    now: replica_sync::ElapsedSinceAppStart,
) -> Vec<replica_sync::state_machine::Output<Op>> {
    let Some(at) = state.member_snapshot.part_requested_at else {
        return Vec::new();
    };
    if (*now).saturating_sub(*at) < replica_sync::SNAPSHOT_PART_REPLY_TIMEOUT {
        return Vec::new();
    }
    state.member_snapshot.part_requested_at = None;
    tracing::debug!(target: "replica_sync", "snapshot part request expired");
    state.retry_fetch()
}

#[cfg(test)]
mod tests {
    use super::expire_overdue_snapshot_part_request;
    use crate::identity;
    use crate::p2p;
    use crate::replica_sync;
    use crate::replica_sync::state_machine::{Input, Output, ReplicaState, ZoneLiveness};

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let now = replica_sync::ElapsedSinceAppStart::default();
        assert_eq!(
            expire_overdue_snapshot_part_request(&mut state, now),
            Vec::new()
        );
    }

    fn at_secs(secs: u64) -> replica_sync::ElapsedSinceAppStart {
        replica_sync::ElapsedSinceAppStart::from(std::time::Duration::from_secs(secs))
    }

    fn alive_status() -> ZoneLiveness {
        ZoneLiveness {
            is_zone_still_listed: true,
            has_other_history_holders: true,
        }
    }

    fn other_zone_remote() -> Input<u8> {
        Input::Remote {
            zone: identity::ZoneName::from("other"),
            meta: replica_sync::OpMeta {
                lamport: 1,
                author: 1,
            },
            op: 1u8,
        }
    }

    #[test]
    fn an_overdue_snapshot_part_request_is_retried() {
        let mut state = ReplicaState::<u8>::default();
        let zone = identity::ZoneName::from("r");
        let first = identity::PeerId::from("a");
        let second = identity::PeerId::from("b");
        let joined = state.update(
            Input::Joined {
                zone: zone.clone(),
                prefix: p2p::HistoryKeyPrefix::from("prefix"),
                members: vec![first.clone(), second.clone()],
                liveness: alive_status(),
            },
            at_secs(0),
        );
        assert_eq!(
            joined,
            vec![Output::FetchSnapshotPartFromZoneMember {
                peer: first,
                zone: zone.clone(),
                part: 0,
            }]
        );
        let outputs = state.update(other_zone_remote(), at_secs(13));
        assert_eq!(
            outputs,
            vec![Output::FetchSnapshotPartFromZoneMember {
                peer: second,
                zone,
                part: 0,
            }]
        );
        assert_eq!(state.member_snapshot.part_requested_at, Some(at_secs(13)));
    }

    #[test]
    fn a_fresh_snapshot_part_request_is_left_alone() {
        let mut state = ReplicaState::<u8>::default();
        let zone = identity::ZoneName::from("r");
        state.update(
            Input::Joined {
                zone,
                prefix: p2p::HistoryKeyPrefix::from("prefix"),
                members: vec![identity::PeerId::from("a"), identity::PeerId::from("b")],
                liveness: alive_status(),
            },
            at_secs(0),
        );
        let outputs = state.update(other_zone_remote(), at_secs(5));
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.member_snapshot.part_requested_at, Some(at_secs(0)));
    }
}
