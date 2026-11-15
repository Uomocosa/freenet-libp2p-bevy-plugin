use std::time::Duration;

use bevy::prelude::MessageWriter;

use crate::identity;
use crate::p2p;
use crate::replica_sync;

pub fn perform<Replica: replica_sync::traits::ReplicatedState>(
    output: replica_sync::state_machine::Output<Replica::Op>,
    room: &mut Replica,
    clock: replica_sync::LamportClock,
    per_room_bytes: p2p::HistoryByteCount,
    record_ttl: Duration,
    pending: &mut Vec<p2p::Libp2pCommand>,
    failures: &mut Option<MessageWriter<replica_sync::JoinFailure>>,
    game: &identity::GameId,
) {
    use p2p::Libp2pCommand;
    use replica_sync::state_machine::Output;
    match output {
        Output::FetchSnapshotPartFromRoomMember { peer, room, part } => {
            pending.push(Libp2pCommand::FetchSnapshot {
                peer_id: peer,
                room,
                part,
            });
        }
        Output::SendSnapshotPartToRoomMember {
            id,
            room: name,
            part,
        } => {
            let frame = replica_sync::bevy_systems::SyncState::<Replica::Op>::snapshot_frame(
                room, name, part,
            );
            pending.push(Libp2pCommand::RespondSnapshot { id, frame });
        }
        Output::Merge { bytes } => {
            if let Ok(snapshot) = bincode::deserialize::<Replica::Snapshot>(&bytes) {
                room.merge(snapshot);
            }
        }
        Output::Apply { meta, op } => room.apply(meta, &op),
        Output::GetHistoryRecord { id } => pending.push(Libp2pCommand::GetHistoryRecord { id }),
        Output::MergeAndWriteHistory {
            prefix,
            stored_manifest,
            stored_snapshot,
        } => {
            pending.extend(p2p::Libp2pCommand::merge_and_write_history_records(
                room,
                &prefix,
                stored_manifest.as_ref(),
                stored_snapshot.as_deref(),
                u64::from(*clock),
                per_room_bytes,
                record_ttl,
            ));
        }
        Output::RoomGone { room } => {
            if let Some(writer) = failures {
                writer.write(replica_sync::JoinFailure::RoomGone(room));
            }
        }
        Output::HistoryUnavailable { room } => {
            if let Some(writer) = failures {
                writer.write(replica_sync::JoinFailure::HistoryUnavailable(room));
            }
        }
        Output::PublishSliceContentHashes { room, hashes } => {
            tracing::debug!(target: "replica_sync", "anti-entropy publish room={room} slices={}", hashes.len());
            let msg =
                replica_sync::ReplicaSyncTopicMessage::<Replica::Op>::CompareSliceContentHashes {
                    room: room.clone(),
                    hashes,
                };
            if let Ok(data) = bincode::serialize(&msg) {
                pending.push(Libp2pCommand::Publish {
                    topic: identity::Topic::from_replica_sync(game, &room),
                    data,
                });
            }
        }
        Output::FetchReplicaSlice { peer, room, index } => {
            pending.push(Libp2pCommand::FetchReplicaSlice {
                peer_id: peer,
                room,
                index,
            });
        }
        Output::SendReplicaSlice {
            id,
            room: name,
            index,
        } => {
            let frame = bincode::serialize(&room.slice_snapshot(index)).map_or_else(
                |_| p2p::SnapshotResponse::Missing { room: name.clone() },
                |data| p2p::SnapshotResponse::ReplicaSlice {
                    room: name.clone(),
                    index,
                    data,
                },
            );
            pending.push(Libp2pCommand::RespondSnapshot { id, frame });
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Resource;

    use super::perform;
    use crate::identity;
    use crate::p2p;
    use crate::replica_sync;

    #[derive(Resource, Default)]
    struct MockReplica;

    impl replica_sync::traits::ReplicatedState for MockReplica {
        type Op = u8;
        type Snapshot = Vec<u8>;

        fn apply(&mut self, _meta: replica_sync::OpMeta, _op: &u8) {}
        fn snapshot(&self) -> Vec<u8> {
            Vec::new()
        }
        fn merge(&mut self, _snapshot: Vec<u8>) {}
        fn slice_content_hashes(&self) -> replica_sync::ReplicaSliceContentHashes {
            Vec::new()
        }
        fn slice_snapshot(&self, _index: replica_sync::ReplicaSliceIndex) -> Vec<u8> {
            Vec::new()
        }
    }

    #[test]
    fn test_usage() {
        let mut room = MockReplica;
        let mut pending = Vec::new();
        let game = identity::GameId::from("game");
        perform(
            replica_sync::state_machine::Output::GetHistoryRecord {
                id: p2p::RoomHistoryRecordKind::Manifest(p2p::HistoryKeyPrefix::from("abc")),
            },
            &mut room,
            replica_sync::LamportClock::default(),
            p2p::HistoryByteCount::from(1_u64),
            std::time::Duration::from_secs(1),
            &mut pending,
            &mut None,
            &game,
        );
        assert_eq!(pending.len(), 1);
    }
}
