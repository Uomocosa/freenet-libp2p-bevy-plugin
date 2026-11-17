use std::collections::BTreeMap;

use crate::p2p;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PendingHistory {
    pub manifest: Option<p2p::HistoryManifest>,
    pub chunks: BTreeMap<p2p::ChunkIndex, Vec<u8>>,
}

// no test_usage necessary
