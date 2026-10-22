use libp2p::kad::RecordKey;

use crate::p2p;

#[must_use]
pub fn parse_history_record_key(record_key: &RecordKey) -> Option<p2p::RoomHistoryRecordKind> {
    let raw = std::str::from_utf8(record_key.as_ref()).ok()?;
    let rest = raw.strip_prefix("lobby/history/")?;
    let (prefix, rest) = rest.split_once('/')?;
    if rest == "manifest" {
        return Some(p2p::RoomHistoryRecordKind::Manifest(
            p2p::HistoryKeyPrefix::from(prefix),
        ));
    }
    let (content_hash, rest) = rest.split_once('/')?;
    let (kind, chunk) = rest.split_once('/')?;
    if kind != "chunk" {
        return None;
    }
    Some(p2p::RoomHistoryRecordKind::Chunk(p2p::HistoryChunkId {
        prefix: p2p::HistoryKeyPrefix::from(prefix),
        content_hash: p2p::ContentHash::from(content_hash),
        chunk: p2p::ChunkIndex(chunk.parse().ok()?),
    }))
}

#[cfg(test)]
mod tests {
    use libp2p::kad::RecordKey;

    use super::parse_history_record_key;

    use crate::p2p;

    fn chunk_id(prefix: &str, hash: &str, chunk: u64) -> p2p::RoomHistoryRecordKind {
        p2p::RoomHistoryRecordKind::Chunk(p2p::HistoryChunkId {
            prefix: p2p::HistoryKeyPrefix::from(prefix),
            content_hash: p2p::ContentHash::from(hash),
            chunk: p2p::ChunkIndex(chunk),
        })
    }

    #[test]
    fn test_usage() {
        let key = chunk_id("abc", "def", 7).history_record_key();
        assert_eq!(
            parse_history_record_key(&key),
            Some(chunk_id("abc", "def", 7))
        );
    }

    #[test]
    fn test_usage_round_trips_the_manifest() {
        let manifest = p2p::RoomHistoryRecordKind::Manifest(p2p::HistoryKeyPrefix::from("abc"));
        let key = manifest.history_record_key();
        assert_eq!(parse_history_record_key(&key), Some(manifest));
    }

    #[test]
    fn test_usage_keys_differ_by_content_hash() {
        let first = chunk_id("abc", "aaa", 0).history_record_key();
        let second = chunk_id("abc", "bbb", 0).history_record_key();
        assert_ne!(first.as_ref(), second.as_ref());
    }

    #[test]
    fn test_usage_rejects_non_history_keys() {
        assert_eq!(
            parse_history_record_key(&RecordKey::new(&"lobby/room/room-a")),
            None
        );
        assert_eq!(
            parse_history_record_key(&RecordKey::new(&"lobby/history/room-a/xx")),
            None
        );
        assert_eq!(
            parse_history_record_key(&RecordKey::new(&"lobby/history/abc/def/other/1")),
            None
        );
        assert_eq!(parse_history_record_key(&RecordKey::new(&"a/b/c/d")), None);
    }
}
