use crate::identity;
use crate::p2p;
use crate::room_sync;

pub fn libp2p_event_to_input<Replica: room_sync::traits::ReplicatedState>(
    event: &p2p::Libp2pEvent,
    state: &room_sync::bevy_systems::SyncState<Replica::Op>,
    clock: &mut room_sync::LamportClock,
    game: &identity::GameId,
    replica: &Replica,
) -> Option<room_sync::state_machine::Input<Replica::Op>> {
    match event {
        p2p::Libp2pEvent::Gossip { topic, from, data } => {
            let expected = state
                .room
                .as_ref()
                .map(|room| identity::Topic::from_room_sync(game, room));
            if expected.as_ref() != Some(topic) {
                return None;
            }
            let message =
                bincode::deserialize::<room_sync::ReplicaSyncTopicMessage<Replica::Op>>(data)
                    .ok()?;
            match message {
                room_sync::ReplicaSyncTopicMessage::ApplyThisOp { room, meta, op } => {
                    clock.observe(meta.lamport);
                    Some(room_sync::state_machine::Input::Remote { room, meta, op })
                }
                room_sync::ReplicaSyncTopicMessage::CompareSliceContentHashes {
                    hashes, ..
                } => {
                    tracing::debug!(target: "room_sync", "anti-entropy compare from={from} remote_slices={}", hashes.len());
                    Some(room_sync::state_machine::Input::RemoteSliceContentHashes {
                        from: from.clone(),
                        local: replica.slice_content_hashes(),
                        remote: hashes,
                    })
                }
            }
        }
        p2p::Libp2pEvent::SnapshotPart {
            room,
            part,
            total,
            data,
            ..
        } => Some(
            room_sync::state_machine::Input::SnapshotPartReceivedFromRoomMember {
                room: room.clone(),
                part: *part,
                total: *total,
                data: data.clone(),
            },
        ),
        p2p::Libp2pEvent::SnapshotMissing { .. } => {
            Some(room_sync::state_machine::Input::SnapshotMissingFromRoomMember)
        }
        p2p::Libp2pEvent::RoomSyncRequestFailed { from, request } => match request {
            p2p::SnapshotRequest::Fetch { .. } => {
                Some(room_sync::state_machine::Input::RequestToRoomMemberFailed {
                    from: from.clone(),
                })
            }
            p2p::SnapshotRequest::FetchReplicaSlice { index, .. } => {
                Some(room_sync::state_machine::Input::ReplicaSliceRequestFailed {
                    from: from.clone(),
                    index: *index,
                })
            }
        },
        p2p::Libp2pEvent::HistoryRecordFound { id, value } => {
            Some(room_sync::state_machine::Input::HistoryRecordFound {
                id: id.clone(),
                value: value.clone(),
            })
        }
        p2p::Libp2pEvent::HistoryRecordNotFound { id } => {
            Some(room_sync::state_machine::Input::HistoryRecordNotFound { id: id.clone() })
        }
        p2p::Libp2pEvent::ReplicaSliceRequest {
            id, room, index, ..
        } => Some(room_sync::state_machine::Input::ReplicaSliceRequested {
            id: *id,
            room: room.clone(),
            index: *index,
        }),
        p2p::Libp2pEvent::ReplicaSlice {
            from,
            room,
            index,
            data,
        } => {
            tracing::debug!(target: "room_sync", "anti-entropy slice from={from} index={} bytes={}", **index, data.len());
            Some(room_sync::state_machine::Input::ReplicaSliceReceived {
                from: from.clone(),
                room: room.clone(),
                index: *index,
                data: data.clone(),
            })
        }
        p2p::Libp2pEvent::SnapshotRequest { id, room, part, .. } => Some(
            room_sync::state_machine::Input::SnapshotPartRequestedByRoomMember {
                id: *id,
                room: room.clone(),
                part: *part,
            },
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Resource;
    use derive_more::{Deref, DerefMut};

    use super::libp2p_event_to_input;
    use crate::identity;
    use crate::p2p;
    use crate::room_sync;

    #[derive(Resource, Default, Deref, DerefMut)]
    struct Probe(u8);

    impl room_sync::traits::ReplicatedState for Probe {
        type Op = u32;
        type Snapshot = u32;

        fn apply(&mut self, _meta: room_sync::OpMeta, _op: &Self::Op) {}

        fn snapshot(&self) -> Self::Snapshot {
            0
        }

        fn merge(&mut self, _snapshot: Self::Snapshot) {}

        fn slice_content_hashes(&self) -> room_sync::ReplicaSliceContentHashes {
            Vec::new()
        }

        fn slice_snapshot(&self, _index: room_sync::ReplicaSliceIndex) -> Self::Snapshot {
            0
        }
    }

    #[test]
    fn test_usage() {
        let room = identity::RoomName::from("room");
        let state =
            room_sync::bevy_systems::SyncState::<u32>(room_sync::state_machine::ReplicaState {
                room: Some(room.clone()),
                ..Default::default()
            });
        let mut clock = room_sync::LamportClock::default();
        let game = identity::GameId::from("game");
        let replica = Probe::default();
        let missing = p2p::Libp2pEvent::SnapshotMissing {
            from: identity::PeerId::from("peer"),
            room,
        };
        assert!(matches!(
            libp2p_event_to_input(&missing, &state, &mut clock, &game, &replica),
            Some(room_sync::state_machine::Input::SnapshotMissingFromRoomMember)
        ));
        let unrelated = p2p::Libp2pEvent::HistoryRecordNotFound {
            id: p2p::RoomHistoryRecordKind::Manifest(p2p::HistoryKeyPrefix::from("prefix")),
        };
        assert!(libp2p_event_to_input(&unrelated, &state, &mut clock, &game, &replica).is_some());
    }
}
