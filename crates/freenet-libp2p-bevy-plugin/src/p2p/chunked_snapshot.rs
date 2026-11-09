use atomic_delegate_macros::atomic_delegates;
use serde::{Deserialize, Serialize};

use crate::p2p;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkedSnapshot {
    pub manifest: p2p::HistoryManifest,
    pub chunks: Vec<Vec<u8>>,
}

#[atomic_delegates]
impl ChunkedSnapshot {
    pub fn try_from_payload(
        payload: &[u8],
        lamport_max: u64,
    ) -> Result<ChunkedSnapshot, p2p::ChunkError> {
    }
    pub fn join(
        chunks: &[Vec<u8>],
        manifest: &p2p::HistoryManifest,
    ) -> Result<Vec<u8>, p2p::ChunkError> {
    }
}

#[cfg(test)]
mod tests {
    use super::ChunkedSnapshot;

    #[test]
    fn test_usage() {
        let split = ChunkedSnapshot::try_from_payload(&[7, 8, 9], 1).expect("split");
        assert_eq!(split.chunks.len(), 1);
    }
}
