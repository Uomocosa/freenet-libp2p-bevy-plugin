use atomic_delegate_macros::atomic_delegates;
use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Deref)]
pub struct ReplicaSliceContentHash(pub u64);

#[atomic_delegates]
impl From<&[u8]> for ReplicaSliceContentHash {
    fn from(bytes: &[u8]) -> Self {}
}
