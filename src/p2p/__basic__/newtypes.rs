use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Deref, From,
)]
#[from(forward)]
pub struct ChunkIndex(pub u64);
