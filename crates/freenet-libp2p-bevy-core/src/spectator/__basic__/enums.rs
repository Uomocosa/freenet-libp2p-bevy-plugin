use crate::p2p;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpectateFreshness {
    #[default]
    Bootstrapping,
    Live,
    Repairing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapOutcome {
    RequestChunks(Vec<p2p::ZoneHistoryRecordKind>),
    Snapshot(Vec<u8>),
    Ignored,
}

// no test_usage necessary
