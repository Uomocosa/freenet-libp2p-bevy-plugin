use libp2p::kad;

use crate::p2p;

pub fn handle_found_record<T: p2p::Message>(
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    peer_record: kad::PeerRecord,
) {
    let record = peer_record.record;
    let Some(id) = p2p::parse_record_key(&record.key) else {
        return;
    };
    event_tx
        .send(p2p::Event::Net(p2p::NetEvent::HistoryChunk {
            id,
            data: record.value,
        }))
        .ok();
}

#[cfg(test)]
mod tests {
    use libp2p::kad;

    use super::handle_found_record;
    use crate::net_id;
    use crate::p2p;

    fn peer_record(key: kad::RecordKey) -> kad::PeerRecord {
        kad::PeerRecord {
            peer: None,
            record: kad::Record {
                key,
                value: vec![1, 2, 3],
                publisher: None,
                expires: None,
            },
        }
    }

    #[test]
    fn test_usage() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event<u32>>();
        let chunk_id = p2p::HistoryChunkId {
            room: net_id::RoomName::from("room-a"),
            chunk: p2p::ChunkIndex(5),
        };
        handle_found_record(&tx, peer_record(p2p::record_key(&chunk_id)));
        let Ok(p2p::Event::Net(p2p::NetEvent::HistoryChunk { id, data })) = rx.try_recv() else {
            panic!("expected a history chunk");
        };
        assert_eq!(
            (id.room.as_str(), *id.chunk, data),
            ("room-a", 5, vec![1, 2, 3])
        );
    }

    #[test]
    fn test_usage_skips_malformed_keys() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event<u32>>();
        handle_found_record(&tx, peer_record(kad::RecordKey::new(&"lobby/history/r/xx")));
        handle_found_record(
            &tx,
            peer_record(p2p::provider_key(&net_id::RoomName::from("room-a"))),
        );
        assert!(rx.try_recv().is_err());
    }
}
