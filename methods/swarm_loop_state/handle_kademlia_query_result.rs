use libp2p::kad;

use crate::p2p;

pub fn handle_kademlia_query_result<Payload: p2p::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    result: kad::QueryResult,
) {
    let Some(event) = p2p::SwarmLoopState::<Payload>::history_event(result) else {
        return;
    };
    state.event_tx.send(p2p::Event::Net(event)).ok();
}

#[cfg(test)]
mod tests {
    use libp2p::kad;

    use crate::p2p;

    fn chunk() -> p2p::RoomHistoryRecordKind {
        p2p::RoomHistoryRecordKind::Chunk(p2p::HistoryChunkId {
            prefix: p2p::HistoryKeyPrefix::from("abc"),
            content_hash: p2p::ContentHash::from("def"),
            chunk: p2p::ChunkIndex(5),
        })
    }

    fn found(key: kad::RecordKey) -> kad::QueryResult {
        kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(kad::PeerRecord {
            peer: None,
            record: kad::Record {
                key,
                value: vec![1, 2, 3],
                publisher: None,
                expires: None,
            },
        })))
    }

    fn next_event(
        state: &mut p2p::SwarmLoopState<u32>,
        rx: &mut tokio::sync::mpsc::UnboundedReceiver<p2p::Event<u32>>,
        result: kad::QueryResult,
    ) -> Option<p2p::NetEvent> {
        state.handle_kademlia_query_result(result);
        match rx.try_recv() {
            Ok(p2p::Event::Net(event)) => Some(event),
            _ => None,
        }
    }

    #[test]
    fn test_usage() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        assert_eq!(
            next_event(&mut state, &mut rx, found(chunk().history_record_key())),
            Some(p2p::NetEvent::HistoryRecordFound {
                id: chunk(),
                value: vec![1, 2, 3]
            })
        );
    }

    #[test]
    fn test_usage_reports_missing_and_stored_records() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        let key = chunk().history_record_key();
        let missing =
            kad::QueryResult::GetRecord(Err(kad::GetRecordError::Timeout { key: key.clone() }));
        let stored = kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key }));
        assert_eq!(
            next_event(&mut state, &mut rx, missing),
            Some(p2p::NetEvent::HistoryRecordNotFound { id: chunk() })
        );
        assert_eq!(
            next_event(&mut state, &mut rx, stored),
            Some(p2p::NetEvent::HistoryRecordStored { id: chunk() })
        );
    }

    #[test]
    fn test_usage_skips_non_history_keys() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        assert_eq!(
            next_event(
                &mut state,
                &mut rx,
                found(kad::RecordKey::new(&"lobby/room/room-a"))
            ),
            None
        );
    }
}
