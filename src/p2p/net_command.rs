use atomic_delegate_macros::atomic_delegates;
use serde::{Deserialize, Serialize};

use crate::identity;
use crate::p2p;
use crate::room_sync;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetCommand {
    Dial(identity::Peer),
    ReserveRelay {
        relay_addr: identity::PeerAddr,
    },
    SetMdns {
        mode: p2p::MdnsMode,
    },
    AddKadPeer(identity::Peer),
    PutHistoryRecord {
        id: p2p::RoomHistoryRecordKind,
        value: Vec<u8>,
        ttl: std::time::Duration,
    },
    GetHistoryRecord {
        id: p2p::RoomHistoryRecordKind,
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
    SubscribeRoom {
        room: identity::RoomName,
    },
    UnsubscribeRoom {
        room: identity::RoomName,
    },
    FetchSnapshot {
        peer_id: identity::PeerId,
        room: identity::RoomName,
        part: u32,
    },
    FetchReplicaSlice {
        peer_id: identity::PeerId,
        room: identity::RoomName,
        index: room_sync::ReplicaSliceIndex,
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
impl NetCommand {
    pub fn merge_and_write_history_records<Replica: room_sync::RoomState>(
        replica: &mut Replica,
        prefix: &p2p::HistoryKeyPrefix,
        stored_manifest: Option<&p2p::HistoryManifest>,
        stored_snapshot: Option<&[u8]>,
        lamport_max: u64,
        per_room_bytes: p2p::HistoryByteCount,
        ttl: std::time::Duration,
    ) -> Vec<p2p::NetCommand> {
    }
}

#[cfg(test)]
mod tests {
    use super::NetCommand;
    use crate::identity;

    #[test]
    fn test_usage() {
        let command = NetCommand::Dial(identity::Peer {
            id: identity::PeerId::from("peer"),
            addrs: Vec::new(),
        });
        assert!(matches!(command, NetCommand::Dial(_)));
    }
}
