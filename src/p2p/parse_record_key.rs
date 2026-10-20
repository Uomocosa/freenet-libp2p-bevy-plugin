use libp2p::kad::RecordKey;

use crate::p2p;

#[must_use]
pub fn parse_record_key(record_key: &RecordKey) -> Option<p2p::HistoryChunkId> {
    let raw = std::str::from_utf8(record_key.as_ref()).ok()?;
    let rest = raw.strip_prefix("lobby/history/")?;
    let (prefix, rest) = rest.split_once('/')?;
    let (content_hash, rest) = rest.split_once('/')?;
    let (kind, chunk) = rest.split_once('/')?;
    if kind != "chunk" {
        return None;
    }
    Some(p2p::HistoryChunkId {
        prefix: p2p::HistoryKeyPrefix::from(prefix),
        content_hash: p2p::ContentHash::from(content_hash),
        chunk: p2p::ChunkIndex(chunk.parse().ok()?),
    })
}

#[cfg(test)]
mod tests {
    use libp2p::kad::RecordKey;

    use super::parse_record_key;
    use crate::p2p;

    fn chunk_id(prefix: &str, hash: &str, chunk: u64) -> p2p::HistoryChunkId {
        p2p::HistoryChunkId {
            prefix: p2p::HistoryKeyPrefix::from(prefix),
            content_hash: p2p::ContentHash::from(hash),
            chunk: p2p::ChunkIndex(chunk),
        }
    }

    #[test]
    fn test_usage() {
        let key = p2p::record_key(&chunk_id("abc", "def", 7));
        assert_eq!(parse_record_key(&key), Some(chunk_id("abc", "def", 7)));
    }

    #[test]
    fn test_usage_keys_differ_by_content_hash() {
        let first = p2p::record_key(&chunk_id("abc", "aaa", 0));
        let second = p2p::record_key(&chunk_id("abc", "bbb", 0));
        assert_ne!(first.as_ref(), second.as_ref());
    }

    #[test]
    fn test_usage_rejects_non_history_keys() {
        assert_eq!(
            parse_record_key(&RecordKey::new(&"lobby/room/room-a")),
            None
        );
        assert_eq!(
            parse_record_key(&RecordKey::new(&"lobby/history/abc/manifest")),
            None
        );
        assert_eq!(
            parse_record_key(&RecordKey::new(&"lobby/history/room-a/xx")),
            None
        );
        assert_eq!(parse_record_key(&RecordKey::new(&"a/b/c/d")), None);
    }
}
