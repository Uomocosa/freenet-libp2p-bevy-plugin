use crate::p2p;

#[must_use]
pub fn history_chunk(prefix: &str, content_hash: &str, chunk: u64) -> p2p::ZoneHistoryRecordKind {
    p2p::ZoneHistoryRecordKind::Chunk(p2p::HistoryChunkId {
        prefix: p2p::HistoryKeyPrefix::from(prefix),
        content_hash: p2p::ContentHash::from(content_hash),
        chunk: p2p::ChunkIndex(chunk),
    })
}

#[cfg(test)]
mod tests {
    use crate::p2p;

    use super::history_chunk;

    #[test]
    fn test_usage() {
        let kind = history_chunk("abc", "def", 7);
        let p2p::ZoneHistoryRecordKind::Chunk(id) = kind else {
            panic!("expected a chunk record");
        };
        assert_eq!(id.prefix, p2p::HistoryKeyPrefix::from("abc"));
        assert_eq!(id.content_hash, p2p::ContentHash::from("def"));
        assert_eq!(id.chunk, p2p::ChunkIndex(7));
    }
}
