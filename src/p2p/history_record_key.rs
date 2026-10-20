use libp2p::kad::RecordKey;

use crate::p2p;

#[must_use]
pub fn history_record_key(id: &p2p::RoomHistoryRecordKind) -> RecordKey {
    match id {
        p2p::RoomHistoryRecordKind::Manifest(prefix) => {
            RecordKey::new(&format!("lobby/history/{}/manifest", prefix.as_str()))
        }
        p2p::RoomHistoryRecordKind::Chunk(chunk) => RecordKey::new(&format!(
            "lobby/history/{}/{}/chunk/{:08}",
            chunk.prefix.as_str(),
            chunk.content_hash.as_str(),
            *chunk.chunk
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::history_record_key;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let prefix = p2p::HistoryKeyPrefix::from("abc");
        let manifest = history_record_key(&p2p::RoomHistoryRecordKind::Manifest(prefix.clone()));
        let chunk = history_record_key(&p2p::RoomHistoryRecordKind::Chunk(p2p::HistoryChunkId {
            prefix,
            content_hash: p2p::ContentHash::from("def"),
            chunk: p2p::ChunkIndex(0),
        }));
        assert_eq!(manifest.as_ref(), b"lobby/history/abc/manifest");
        assert_eq!(chunk.as_ref(), b"lobby/history/abc/def/chunk/00000000");
    }
}
