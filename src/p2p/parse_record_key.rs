use libp2p::kad::RecordKey;

use crate::net_id;
use crate::p2p;

#[must_use]
pub fn parse_record_key(record_key: &RecordKey) -> Option<p2p::HistoryChunkId> {
    let raw = std::str::from_utf8(record_key.as_ref()).ok()?;
    let rest = raw.strip_prefix("lobby/history/")?;
    let (room, chunk) = rest.rsplit_once('/')?;
    Some(p2p::HistoryChunkId {
        room: net_id::RoomName::from(room),
        chunk: p2p::ChunkIndex(chunk.parse().ok()?),
    })
}

#[cfg(test)]
mod tests {
    use libp2p::kad::RecordKey;

    use super::parse_record_key;
    use crate::net_id;
    use crate::p2p;

    fn chunk_id(room: &str, chunk: u64) -> p2p::HistoryChunkId {
        p2p::HistoryChunkId {
            room: net_id::RoomName::from(room),
            chunk: p2p::ChunkIndex(chunk),
        }
    }

    #[test]
    fn test_usage() {
        let key = p2p::record_key(&chunk_id("room-a", 7));
        assert_eq!(parse_record_key(&key), Some(chunk_id("room-a", 7)));
    }

    #[test]
    fn test_usage_room_with_slash_round_trips() {
        let key = p2p::record_key(&chunk_id("team/blue", 3));
        assert_eq!(parse_record_key(&key), Some(chunk_id("team/blue", 3)));
    }

    #[test]
    fn test_usage_rejects_non_history_keys() {
        assert_eq!(
            parse_record_key(&RecordKey::new(&"lobby/room/room-a")),
            None
        );
        assert_eq!(
            parse_record_key(&RecordKey::new(&"lobby/history/room-a/xx")),
            None
        );
        assert_eq!(parse_record_key(&RecordKey::new(&"a/b/c/d")), None);
    }
}
