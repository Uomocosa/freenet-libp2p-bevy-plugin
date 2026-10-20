use libp2p::kad::RecordKey;

use crate::p2p;

#[must_use]
pub fn record_key(id: &p2p::HistoryChunkId) -> RecordKey {
    RecordKey::new(&format!(
        "lobby/history/{}/{}/chunk/{:08}",
        id.prefix.as_str(),
        id.content_hash.as_str(),
        *id.chunk
    ))
}

#[cfg(test)]
mod tests {
    use super::record_key;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let key = record_key(&p2p::HistoryChunkId {
            prefix: p2p::HistoryKeyPrefix::from("abc"),
            content_hash: p2p::ContentHash::from("def"),
            chunk: p2p::ChunkIndex(0),
        });
        assert_ne!(key.as_ref().len(), 0);
    }
}
