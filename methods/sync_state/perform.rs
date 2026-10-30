use std::time::Duration;

use bevy::prelude::ResMut;

use crate::discovery;
use crate::identity;
use crate::p2p;
use crate::room_sync;

pub fn perform<Replica: room_sync::RoomState>(
    output: room_sync::state_machine::Output<Replica::Op>,
    room: &mut Replica,
    clock: room_sync::LamportClock,
    per_room_bytes: p2p::HistoryByteCount,
    record_ttl: Duration,
    pending: &mut p2p::PendingNetCommands,
    command_sender: Option<&discovery::CommandSender>,
    join_error: &mut Option<ResMut<discovery::ui::JoinError>>,
    game: &identity::GameId,
) {
    use p2p::NetCommand;
    use room_sync::state_machine::Output;
    match output {
        Output::FetchSnapshotPartFromRoomMember { peer, room, part } => {
            pending.push(NetCommand::FetchSnapshot {
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
            let frame =
                room_sync::bevy_systems::SyncState::<Replica::Op>::snapshot_frame(room, name, part);
            pending.push(NetCommand::RespondSnapshot { id, frame });
        }
        Output::Merge { bytes } => {
            if let Ok(snapshot) = bincode::deserialize::<Replica::Snapshot>(&bytes) {
                room.merge(snapshot);
            }
        }
        Output::Apply { meta, op } => room.apply(meta, &op),
        Output::GetHistoryRecord { id } => pending.push(NetCommand::GetHistoryRecord { id }),
        Output::MergeAndWriteHistory {
            prefix,
            stored_manifest,
            stored_snapshot,
        } => {
            pending.extend(p2p::NetCommand::merge_and_write_history_records(
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
            if let Some(err) = join_error {
                ***err = Some(format!(
                    "Room {} no longer exists (all players left)",
                    room.as_str()
                ));
            }
            if let Some(sender) = command_sender {
                sender.send(discovery::Command::Leave).ok();
            }
        }
        Output::HistoryUnavailable { room: _ } => {
            if let Some(err) = join_error {
                ***err = Some(String::from("Couldn't get room history"));
            }
            if let Some(sender) = command_sender {
                sender.send(discovery::Command::Leave).ok();
            }
        }
        Output::PublishSliceContentHashes { room, hashes } => {
            tracing::debug!(target: "room_sync", "anti-entropy publish room={room} slices={}", hashes.len());
            let msg =
                room_sync::ReplicaSyncTopicMessage::<Replica::Op>::CompareSliceContentHashes {
                    room: room.clone(),
                    hashes,
                };
            if let Ok(data) = bincode::serialize(&msg) {
                pending.push(NetCommand::Publish {
                    topic: identity::Topic::from_room_sync(game, &room),
                    data,
                });
            }
        }
        Output::FetchReplicaSlice { peer, room, index } => {
            pending.push(NetCommand::FetchReplicaSlice {
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
            pending.push(NetCommand::RespondSnapshot { id, frame });
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Resource;

    use super::perform;
    use crate::identity;
    use crate::p2p;
    use crate::room_sync;

    #[derive(Resource, Default)]
    struct MockReplica;

    impl room_sync::RoomState for MockReplica {
        type Op = u8;
        type Snapshot = Vec<u8>;

        fn apply(&mut self, _meta: room_sync::OpMeta, _op: &u8) {}
        fn snapshot(&self) -> Vec<u8> {
            Vec::new()
        }
        fn merge(&mut self, _snapshot: Vec<u8>) {}
        fn slice_content_hashes(&self) -> room_sync::ReplicaSliceContentHashes {
            Vec::new()
        }
        fn slice_snapshot(&self, _index: room_sync::ReplicaSliceIndex) -> Vec<u8> {
            Vec::new()
        }
    }

    #[test]
    fn test_usage() {
        let mut room = MockReplica;
        let mut pending = p2p::PendingNetCommands::default();
        let game = identity::GameId::from("game");
        perform(
            room_sync::state_machine::Output::GetHistoryRecord {
                id: p2p::RoomHistoryRecordKind::Manifest(p2p::HistoryKeyPrefix::from("abc")),
            },
            &mut room,
            room_sync::LamportClock::default(),
            p2p::HistoryByteCount::from(1_u64),
            std::time::Duration::from_secs(1),
            &mut pending,
            None,
            &mut None,
            &game,
        );
        assert_eq!(pending.0.len(), 1);
    }
}
