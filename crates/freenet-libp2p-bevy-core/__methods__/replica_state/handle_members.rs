use crate::identity;
use crate::p2p;
use crate::replica_sync;

pub fn handle_members<Op>(
    state: &mut replica_sync::state_machine::ReplicaState<Op>,
    members: Vec<identity::PeerId>,
    prefix: p2p::HistoryKeyPrefix,
    liveness: replica_sync::state_machine::ZoneLiveness,
) -> Vec<replica_sync::state_machine::Output<Op>> {
    let mut members = members;
    members.sort();
    state.member_snapshot.members = members;
    state.kademlia_history.prefix = Some(prefix);
    state.kademlia_history.liveness = liveness;
    match state.phase {
        replica_sync::state_machine::Phase::FetchingSnapshotFromZoneMember => {
            if !liveness.is_zone_still_listed {
                let Some(zone) = state.zone.clone() else {
                    return Vec::new();
                };
                state.phase = replica_sync::state_machine::Phase::Failed;
                state.kademlia_history.pending_fetch = None;
                return vec![replica_sync::state_machine::Output::ZoneGone { zone }];
            }
            state.fetch_next()
        }
        replica_sync::state_machine::Phase::FetchingHistoryFromKademlia => {
            if !liveness.is_zone_still_listed {
                let Some(zone) = state.zone.clone() else {
                    return Vec::new();
                };
                state.phase = replica_sync::state_machine::Phase::Failed;
                state.kademlia_history.pending_fetch = None;
                return vec![replica_sync::state_machine::Output::ZoneGone { zone }];
            }
            let outputs = state.fetch_next();
            if !outputs.is_empty() {
                state.phase = replica_sync::state_machine::Phase::FetchingSnapshotFromZoneMember;
                state.kademlia_history.pending_fetch = None;
                state.kademlia_history.fetch_retries = 0;
            }
            outputs
        }
        replica_sync::state_machine::Phase::Ready => {
            if !liveness.is_zone_still_listed {
                let Some(zone) = state.zone.clone() else {
                    return Vec::new();
                };
                state.phase = replica_sync::state_machine::Phase::Failed;
                state.kademlia_history.pending_fetch = None;
                return vec![replica_sync::state_machine::Output::ZoneGone { zone }];
            }
            Vec::new()
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::handle_members;
    use crate::identity;
    use crate::p2p;
    use crate::replica_sync::state_machine::{
        Output, PendingHistoryFetch, Phase, ReplicaState, ZoneLiveness,
    };

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let outputs = handle_members(
            &mut state,
            vec![identity::PeerId::from("peer")],
            p2p::HistoryKeyPrefix::from("prefix"),
            ZoneLiveness::default(),
        );
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.member_snapshot.members.len(), 1);
    }

    #[test]
    fn a_member_connecting_during_the_kademlia_fetch_is_asked_for_the_board() {
        let mut state = ReplicaState::<u8>::default();
        let zone = identity::ZoneName::from("r");
        let prefix = p2p::HistoryKeyPrefix::from("prefix");
        let painter = identity::PeerId::from("painter");
        state.zone = Some(zone.clone());
        state.phase = Phase::FetchingHistoryFromKademlia;
        state.kademlia_history.is_enabled = true;
        state.kademlia_history.fetch_retries = 2;
        state.kademlia_history.pending_fetch = Some(PendingHistoryFetch {
            prefix: prefix.clone(),
            manifest: None,
            received_chunks: BTreeMap::new(),
            records_requested_at: None,
        });
        let outputs = handle_members(
            &mut state,
            vec![painter.clone()],
            prefix,
            ZoneLiveness {
                is_zone_still_listed: true,
                has_other_history_holders: true,
            },
        );
        assert_eq!(
            outputs,
            vec![Output::FetchSnapshotPartFromZoneMember {
                peer: painter,
                zone,
                part: 0,
            }]
        );
        assert_eq!(state.phase, Phase::FetchingSnapshotFromZoneMember);
        assert_eq!(state.kademlia_history.pending_fetch, None);
        assert_eq!(state.kademlia_history.fetch_retries, 0);
    }

    #[test]
    fn a_ready_joiner_alone_sees_the_zone_as_gone() {
        let mut state = ReplicaState::<u8>::default();
        let zone = identity::ZoneName::from("r");
        state.zone = Some(zone.clone());
        state.phase = Phase::Ready;
        let outputs = handle_members(
            &mut state,
            Vec::new(),
            p2p::HistoryKeyPrefix::from("prefix"),
            ZoneLiveness {
                is_zone_still_listed: false,
                has_other_history_holders: false,
            },
        );
        assert_eq!(outputs, vec![Output::ZoneGone { zone }]);
        assert_eq!(state.phase, Phase::Failed);
    }
}
