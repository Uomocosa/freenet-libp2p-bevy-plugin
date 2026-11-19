use bevy::prelude::Resource;

use crate::p2p;
use crate::replica_sync;

#[derive(Resource)]
pub struct FakeReplica<Op, Snapshot> {
    pub stored_snapshot: Snapshot,
    pub applied: Vec<(replica_sync::OpMeta, Op)>,
}

impl<Op, Snapshot: Default> Default for FakeReplica<Op, Snapshot> {
    fn default() -> Self {
        Self {
            stored_snapshot: Snapshot::default(),
            applied: Vec::new(),
        }
    }
}

#[rustfmt::skip]
impl<Op, Snapshot> replica_sync::traits::ReplicatedState for FakeReplica<Op, Snapshot>
where
    Op: p2p::traits::Message,
    Snapshot: p2p::traits::Message + Default,
{
    type Op = Op;
    type Snapshot = Snapshot;

    fn apply(&mut self, meta: replica_sync::OpMeta, op: &Op) { self.applied.push((meta, op.clone())); }

    fn snapshot(&self) -> Snapshot { self.stored_snapshot.clone() }

    fn merge(&mut self, snapshot: Snapshot) { self.stored_snapshot = snapshot; }

    fn slice_content_hashes(&self) -> replica_sync::ReplicaSliceContentHashes { Vec::new() }

    fn slice_snapshot(&self, _index: replica_sync::ReplicaSliceIndex) -> Snapshot { self.stored_snapshot.clone() }
}

#[cfg(test)]
mod tests {
    use crate::replica_sync;
    use crate::replica_sync::traits::ReplicatedState;

    use super::FakeReplica;

    #[test]
    fn test_usage() {
        let mut replica = FakeReplica::<u8, Vec<u8>>::default();
        replica.merge(vec![1, 2, 3]);
        replica.apply(
            replica_sync::OpMeta {
                lamport: 1,
                author: 1,
            },
            &7,
        );
        assert_eq!(replica.snapshot(), vec![1, 2, 3]);
        assert_eq!(replica.applied.len(), 1);
        assert_eq!(replica.slice_content_hashes().len(), 0);
        assert_eq!(
            replica.slice_snapshot(replica_sync::ReplicaSliceIndex(0)),
            vec![1, 2, 3]
        );
    }
}
