use crate::identity;
use crate::p2p;
use crate::replica_sync;
use crate::time;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpectatePhase {
    #[default]
    Idle,
    FetchingSnapshot,
    Live,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpectateInput<Op, Snap> {
    Snapshot {
        history_prefix: p2p::HistoryKeyPrefix,
        snapshot: Snap,
    },
    Topic(replica_sync::ReplicaSyncTopicMessage<Op>),
    RemoteSliceContentHashes {
        from: identity::PeerId,
        zone: identity::ZoneName,
        local: replica_sync::ReplicaSliceContentHashes,
        remote: replica_sync::ReplicaSliceContentHashes,
    },
    SliceReceived {
        zone: identity::ZoneName,
        index: replica_sync::ReplicaSliceIndex,
        snapshot: Snap,
    },
    Players(Vec<identity::PeerId>),
    LocalPeer(Option<identity::PeerId>),
    Tick,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpectateOutput<Op, Snap> {
    Subscribe(identity::Topic),
    Unsubscribe(identity::Topic),
    WatchZone(identity::ZoneName),
    UnwatchZone(identity::ZoneName),
    GetHistoryRecord(p2p::ZoneHistoryRecordKind),
    RequestSlice {
        from: identity::PeerId,
        index: replica_sync::ReplicaSliceIndex,
    },
    Publish(replica_sync::ReplicaSyncTopicMessage<Op>),
    PublishLease {
        zone: identity::ZoneName,
        spectator: identity::PeerId,
        until: time::UnixTime,
    },
    Merge(Snap),
    Apply {
        meta: replica_sync::OpMeta,
        op: Op,
    },
    RepairFinished,
}

// no test_usage necessary
