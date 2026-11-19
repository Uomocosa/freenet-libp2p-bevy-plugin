use std::time::Duration;

use bevy::prelude::MessageWriter;

use crate::identity;
use crate::p2p;
use crate::replica_sync;

pub fn perform<Replica: replica_sync::traits::ReplicatedState>(
    output: replica_sync::state_machine::Output<Replica::Op>,
    zone: &mut Replica,
    clock: replica_sync::LamportClock,
    per_zone_bytes: p2p::HistoryByteCount,
    record_ttl: Duration,
    pending: &mut Vec<p2p::Libp2pCommand>,
    failures: &mut Option<MessageWriter<replica_sync::JoinFailure>>,
    game: &identity::GameId,
) {
    use p2p::Libp2pCommand;
    use replica_sync::state_machine::Output;
    match output {
        Output::FetchSnapshotPartFromZoneMember { peer, zone, part } => {
            pending.push(Libp2pCommand::FetchSnapshot {
                peer_id: peer,
                zone,
                part,
            });
        }
        Output::SendSnapshotPartToZoneMember {
            id,
            zone: name,
            part,
        } => {
            let frame = replica_sync::bevy_systems::SyncState::<Replica::Op>::snapshot_frame(
                zone, name, part,
            );
            pending.push(Libp2pCommand::RespondSnapshot { id, frame });
        }
        Output::Merge { bytes } => {
            if let Ok(snapshot) = bincode::deserialize::<Replica::Snapshot>(&bytes) {
                zone.merge(snapshot);
            }
        }
        Output::Apply { meta, op } => zone.apply(meta, &op),
        Output::GetHistoryRecord { id } => pending.push(Libp2pCommand::GetHistoryRecord { id }),
        Output::MergeAndWriteHistory {
            prefix,
            stored_manifest,
            stored_snapshot,
        } => {
            pending.extend(p2p::Libp2pCommand::merge_and_write_history_records(
                zone,
                &prefix,
                stored_manifest.as_ref(),
                stored_snapshot.as_deref(),
                u64::from(*clock),
                per_zone_bytes,
                record_ttl,
            ));
        }
        Output::ZoneGone { zone } => {
            if let Some(writer) = failures {
                writer.write(replica_sync::JoinFailure::ZoneGone(zone));
            }
        }
        Output::HistoryUnavailable { zone } => {
            if let Some(writer) = failures {
                writer.write(replica_sync::JoinFailure::HistoryUnavailable(zone));
            }
        }
        Output::PublishSliceContentHashes { zone, hashes } => {
            tracing::debug!(target: "replica_sync", "anti-entropy publish zone={zone} slices={}", hashes.len());
            let msg =
                replica_sync::ReplicaSyncTopicMessage::<Replica::Op>::CompareSliceContentHashes {
                    zone: zone.clone(),
                    hashes,
                };
            if let Ok(data) = bincode::serialize(&msg) {
                pending.push(Libp2pCommand::Publish {
                    topic: identity::Topic::from_replica_sync(game, &zone),
                    data,
                });
            }
        }
        Output::FetchReplicaSlice { peer, zone, index } => {
            pending.push(Libp2pCommand::FetchReplicaSlice {
                peer_id: peer,
                zone,
                index,
            });
        }
        Output::SendReplicaSlice {
            id,
            zone: name,
            index,
        } => {
            let frame = bincode::serialize(&zone.slice_snapshot(index)).map_or_else(
                |_| p2p::SnapshotResponse::Missing { zone: name.clone() },
                |data| p2p::SnapshotResponse::ReplicaSlice {
                    zone: name.clone(),
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
    use super::perform;
    use crate::identity;
    use crate::p2p;
    use crate::replica_sync;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut zone = testing::FakeReplica::<u8, Vec<u8>>::default();
        let mut pending = Vec::new();
        let game = identity::GameId::from("game");
        perform(
            replica_sync::state_machine::Output::GetHistoryRecord {
                id: p2p::ZoneHistoryRecordKind::Manifest(p2p::HistoryKeyPrefix::from("abc")),
            },
            &mut zone,
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
