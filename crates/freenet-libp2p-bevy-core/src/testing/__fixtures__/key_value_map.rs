use std::collections::BTreeMap;

use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::replica_sync;
use crate::testing;

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct KeyValueMap {
    pub entries: BTreeMap<u8, u32>,
}

impl replica_sync::traits::ReplicatedState for KeyValueMap {
    type Op = testing::KeyValueOp;
    type Snapshot = Vec<(u8, u32)>;

    fn apply(&mut self, _meta: replica_sync::OpMeta, op: &Self::Op) {
        let testing::KeyValueOp { key, value } = *op;
        self.entries.insert(key, value);
    }

    fn snapshot(&self) -> Self::Snapshot {
        let entries = self.entries.clone();
        entries.into_iter().collect()
    }

    fn merge(&mut self, snapshot: Self::Snapshot) {
        for (key, value) in snapshot {
            self.entries.insert(key, value);
        }
    }

    fn slice_content_hashes(&self) -> replica_sync::ReplicaSliceContentHashes {
        let hash = u64::try_from(self.entries.len()).unwrap_or_default();
        vec![(
            replica_sync::ReplicaSliceIndex(0),
            replica_sync::ReplicaSliceContentHash(hash),
        )]
    }

    fn slice_snapshot(&self, _index: replica_sync::ReplicaSliceIndex) -> Self::Snapshot {
        let entries = self.entries.clone();
        entries.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::replica_sync;
    use crate::replica_sync::traits::ReplicatedState;
    use crate::testing;

    use super::KeyValueMap;

    #[test]
    fn test_usage() {
        let mut map = KeyValueMap::default();
        map.entries.insert(1, 10);
        assert_eq!(map.snapshot(), vec![(1, 10)]);
        map.merge(vec![(2, 20)]);
        assert_eq!(map.entries.get(&2), Some(&20));
        map.apply(
            replica_sync::OpMeta {
                lamport: 1,
                author: 1,
            },
            &testing::KeyValueOp { key: 3, value: 30 },
        );
        assert_eq!(map.entries.get(&3), Some(&30));
        assert_eq!(map.slice_content_hashes().len(), 1);
        assert_eq!(
            map.slice_snapshot(replica_sync::ReplicaSliceIndex(0)),
            vec![(1, 10), (2, 20), (3, 30)]
        );
    }
}
