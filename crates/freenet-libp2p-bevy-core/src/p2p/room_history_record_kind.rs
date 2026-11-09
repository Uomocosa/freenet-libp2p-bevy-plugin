use atomic_delegate_macros::atomic_delegates;
use libp2p::kad::RecordKey;
use serde::{Deserialize, Serialize};

use crate::p2p;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomHistoryRecordKind {
    Manifest(p2p::HistoryKeyPrefix),
    Chunk(p2p::HistoryChunkId),
}

#[atomic_delegates]
impl RoomHistoryRecordKind {
    pub fn parse_history_record_key(record_key: &RecordKey) -> Option<RoomHistoryRecordKind> {}
}

#[rustfmt::skip]
impl RoomHistoryRecordKind {
    #[must_use]
    pub fn history_record_key(&self) -> RecordKey {
        match self {
            Self::Manifest(prefix) => {
                RecordKey::new(&format!("lobby/history/{}/manifest", prefix.as_str()))
            }
            Self::Chunk(chunk) => RecordKey::new(&format!(
                "lobby/history/{}/{}/chunk/{:08}",
                chunk.prefix.as_str(),
                chunk.content_hash.as_str(),
                *chunk.chunk
            )),
        }
    }
    #[must_use] pub(crate) const fn record_prefix(&self) -> &p2p::HistoryKeyPrefix {
        match self {
            Self::Manifest(prefix) => prefix,
            Self::Chunk(chunk_id) => &chunk_id.prefix,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RoomHistoryRecordKind;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let manifest = RoomHistoryRecordKind::Manifest(p2p::HistoryKeyPrefix::from("abc"));
        assert!(
            RoomHistoryRecordKind::history_record_key(&manifest)
                .as_ref()
                .starts_with(b"lobby/")
        );
    }
}
