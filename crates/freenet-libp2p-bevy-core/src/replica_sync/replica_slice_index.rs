use atomic_delegate_macros::atomic_delegates;
use derive_more::Deref;
use serde::{Deserialize, Serialize};

use crate::replica_sync;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Deref,
)]
pub struct ReplicaSliceIndex(pub u32);

#[atomic_delegates]
impl ReplicaSliceIndex {
    pub fn from_hash_mismatches(
        local: &replica_sync::ReplicaSliceContentHashes,
        remote: &replica_sync::ReplicaSliceContentHashes,
    ) -> Vec<Self> {
    }
}

// no test_usage necessary
