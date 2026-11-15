use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Message;
use serde::{Deserialize, Serialize};

use crate::identity;
use crate::p2p;
use crate::replica_sync;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Message)]
pub enum Libp2pCommand {
    Dial(identity::Peer),
    ReserveRelay {
        relay_addr: identity::PeerAddr,
    },
    SetMdns {
        mode: p2p::MdnsMode,
    },
    AddKadPeer(identity::Peer),
    PutHistoryRecord {
        id: p2p::ZoneHistoryRecordKind,
        value: Vec<u8>,
        ttl: std::time::Duration,
    },
    GetHistoryRecord {
        id: p2p::ZoneHistoryRecordKind,
    },
    MeasureHistoryStore,
    RemoveHistoryRecords {
        prefix: p2p::HistoryKeyPrefix,
    },
    Subscribe {
        topic: identity::Topic,
    },
    Unsubscribe {
        topic: identity::Topic,
    },
    SubscribeZone {
        zone: identity::ZoneName,
    },
    UnsubscribeZone {
        zone: identity::ZoneName,
    },
    FetchSnapshot {
        peer_id: identity::PeerId,
        zone: identity::ZoneName,
        part: u32,
    },
    FetchReplicaSlice {
        peer_id: identity::PeerId,
        zone: identity::ZoneName,
        index: replica_sync::ReplicaSliceIndex,
    },
    RespondSnapshot {
        id: p2p::ReplyId,
        frame: p2p::SnapshotResponse,
    },
    Publish {
        topic: identity::Topic,
        data: Vec<u8>,
    },
    PeerExchange {
        peer_id: identity::PeerId,
        data: Vec<u8>,
    },
}

#[atomic_delegates]
impl Libp2pCommand {
    pub fn merge_and_write_history_records<Replica: replica_sync::traits::ReplicatedState>(
        replica: &mut Replica,
        prefix: &p2p::HistoryKeyPrefix,
        stored_manifest: Option<&p2p::HistoryManifest>,
        stored_snapshot: Option<&[u8]>,
        lamport_max: u64,
        per_zone_bytes: p2p::HistoryByteCount,
        ttl: std::time::Duration,
    ) -> Vec<p2p::Libp2pCommand> {
    }
}

#[cfg(test)]
mod tests {
    use super::Libp2pCommand;
    use crate::identity;

    #[test]
    fn test_usage() {
        let command = Libp2pCommand::Dial(identity::Peer {
            id: identity::PeerId::from("peer"),
            addrs: Vec::new(),
        });
        assert!(matches!(command, Libp2pCommand::Dial(_)));
    }
}
