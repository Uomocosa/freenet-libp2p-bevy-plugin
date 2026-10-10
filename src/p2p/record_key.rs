use libp2p::kad::RecordKey;

use crate::p2p;

#[must_use]
pub fn record_key(id: &p2p::HistoryChunkId) -> RecordKey {
    RecordKey::new(&format!(
        "lobby/history/{}/{:08}",
        id.room.as_str(),
        *id.chunk
    ))
}

#[cfg(test)]
mod tests {
    use super::record_key;
    use crate::net_id;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let key = record_key(&p2p::HistoryChunkId {
            room: net_id::RoomName::from("room-a"),
            chunk: p2p::ChunkIndex(0),
        });
        assert_ne!(key.as_ref().len(), 0);
    }
}
