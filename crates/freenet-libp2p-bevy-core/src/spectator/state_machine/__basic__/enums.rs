use crate::identity;
use crate::p2p;
use crate::replica_sync;
use crate::spectator;
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
    Command(spectator::state_machine::SpectateCommand),
    Snapshot(Snap),
    Topic(replica_sync::ReplicaSyncTopicMessage<Op>),
    RemoteSliceContentHashes {
        from: identity::PeerId,
        zone: identity::ZoneName,
        local: replica_sync::ReplicaSliceContentHashes,
        remote: replica_sync::ReplicaSliceContentHashes,
    },
    SliceReceived {
        index: replica_sync::ReplicaSliceIndex,
        snapshot: Snap,
    },
    Players(Vec<identity::PeerId>),
    Lease {
        zone: identity::ZoneName,
        spectator: identity::PeerId,
        until: time::UnixTime,
    },
    Tick,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpectateOutput<Op, Snap> {
    Subscribe(identity::Topic),
    Unsubscribe(identity::Topic),
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
    Clear,
    SpectatorCountChanged(usize),
}

// no test_usage necessary
