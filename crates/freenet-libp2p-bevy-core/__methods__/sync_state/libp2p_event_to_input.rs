use crate::identity;
use crate::p2p;
use crate::replica_sync;

pub fn libp2p_event_to_input<Replica: replica_sync::traits::ReplicatedState>(
    event: &p2p::Libp2pEvent,
    state: &replica_sync::bevy_systems::SyncState<Replica::Op>,
    clock: &mut replica_sync::LamportClock,
    game: &identity::GameId,
    replica: &Replica,
) -> Option<replica_sync::state_machine::Input<Replica::Op>> {
    match event {
        p2p::Libp2pEvent::Gossip { topic, from, data } => {
            let expected = state
                .zone
                .as_ref()
                .map(|zone| identity::Topic::from_replica_sync(game, zone));
            if expected.as_ref() != Some(topic) {
                return None;
            }
            let message =
                bincode::deserialize::<replica_sync::ReplicaSyncTopicMessage<Replica::Op>>(data)
                    .ok()?;
            replica_sync::bevy_systems::SyncState::<Replica::Op>::topic_message_to_input(
                message, from, clock, replica,
            )
        }
        p2p::Libp2pEvent::SnapshotPart {
            zone,
            part,
            total,
            data,
            ..
        } => Some(
            replica_sync::state_machine::Input::SnapshotPartReceivedFromZoneMember {
                zone: zone.clone(),
                part: *part,
                total: *total,
                data: data.clone(),
            },
        ),
        p2p::Libp2pEvent::SnapshotMissing { .. } => {
            Some(replica_sync::state_machine::Input::SnapshotMissingFromZoneMember)
        }
        p2p::Libp2pEvent::ReplicaSyncRequestFailed { from, request } => match request {
            p2p::SnapshotRequest::Fetch { .. } => Some(
                replica_sync::state_machine::Input::RequestToZoneMemberFailed {
                    from: from.clone(),
                },
            ),
            p2p::SnapshotRequest::FetchReplicaSlice { index, .. } => Some(
                replica_sync::state_machine::Input::ReplicaSliceRequestFailed {
                    from: from.clone(),
                    index: *index,
                },
            ),
        },
        p2p::Libp2pEvent::HistoryRecordFound { id, value } => {
            Some(replica_sync::state_machine::Input::HistoryRecordFound {
                id: id.clone(),
                value: value.clone(),
            })
        }
        p2p::Libp2pEvent::HistoryRecordNotFound { id } => {
            Some(replica_sync::state_machine::Input::HistoryRecordNotFound { id: id.clone() })
        }
        p2p::Libp2pEvent::ReplicaSliceRequest {
            id, zone, index, ..
        } => Some(replica_sync::state_machine::Input::ReplicaSliceRequested {
            id: *id,
            zone: zone.clone(),
            index: *index,
        }),
        p2p::Libp2pEvent::ReplicaSlice {
            from,
            zone,
            index,
            data,
        } => {
            tracing::debug!(target: "replica_sync", "anti-entropy slice from={from} index={} bytes={}", **index, data.len());
            Some(replica_sync::state_machine::Input::ReplicaSliceReceived {
                from: from.clone(),
                zone: zone.clone(),
                index: *index,
                data: data.clone(),
            })
        }
        p2p::Libp2pEvent::SnapshotRequest { id, zone, part, .. } => Some(
            replica_sync::state_machine::Input::SnapshotPartRequestedByZoneMember {
                id: *id,
                zone: zone.clone(),
                part: *part,
            },
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::libp2p_event_to_input;
    use crate::identity;
    use crate::p2p;
    use crate::replica_sync;
    use crate::testing;

    #[test]
    fn test_usage() {
        let zone = identity::ZoneName::from("zone");
        let state = replica_sync::bevy_systems::SyncState::<u32>(
            replica_sync::state_machine::ReplicaState {
                zone: Some(zone.clone()),
                ..Default::default()
            },
        );
        let mut clock = replica_sync::LamportClock::default();
        let game = identity::GameId::from("game");
        let replica = testing::FakeReplica::<u32, u32>::default();
        let missing = p2p::Libp2pEvent::SnapshotMissing {
            from: identity::PeerId::from("peer"),
            zone,
        };
        assert!(matches!(
            libp2p_event_to_input(&missing, &state, &mut clock, &game, &replica),
            Some(replica_sync::state_machine::Input::SnapshotMissingFromZoneMember)
        ));
        let unrelated = p2p::Libp2pEvent::HistoryRecordNotFound {
            id: p2p::ZoneHistoryRecordKind::Manifest(p2p::HistoryKeyPrefix::from("prefix")),
        };
        assert!(libp2p_event_to_input(&unrelated, &state, &mut clock, &game, &replica).is_some());
    }
}
