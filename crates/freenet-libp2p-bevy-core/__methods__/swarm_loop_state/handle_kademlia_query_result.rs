use crate::p2p;
use libp2p::kad;

pub fn handle_kademlia_query_result<Payload: p2p::traits::Message>(
    state: &p2p::SwarmLoopState<Payload>,
    result: kad::QueryResult,
) {
    let Some(event) = p2p::SwarmLoopState::<Payload>::history_event(result) else {
        return;
    };
    state.event_tx.send(p2p::Event::Libp2p(event)).ok();
}

#[cfg(test)]
mod tests {
    use libp2p::kad;

    use crate::p2p;
    use crate::testing;

    #[test]
    fn test_usage() {
        let (state, mut rx) = testing::swarm_loop_state::<u32>().expect("swarm loop state");
        let chunk = testing::history_chunk("abc", "def", 5);
        assert_eq!(
            testing::libp2p_event_from_kademlia_result(
                &state,
                &mut rx,
                testing::kademlia_record_found(chunk.history_record_key())
            ),
            Some(p2p::Libp2pEvent::HistoryRecordFound {
                id: chunk,
                value: vec![1, 2, 3]
            })
        );
    }

    #[test]
    fn test_usage_reports_missing_and_stored_records() {
        let (state, mut rx) = testing::swarm_loop_state::<u32>().expect("swarm loop state");
        let chunk = testing::history_chunk("abc", "def", 5);
        let key = chunk.history_record_key();
        let missing =
            kad::QueryResult::GetRecord(Err(kad::GetRecordError::Timeout { key: key.clone() }));
        let stored = kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key }));
        assert_eq!(
            testing::libp2p_event_from_kademlia_result(&state, &mut rx, missing),
            Some(p2p::Libp2pEvent::HistoryRecordNotFound { id: chunk.clone() })
        );
        assert_eq!(
            testing::libp2p_event_from_kademlia_result(&state, &mut rx, stored),
            Some(p2p::Libp2pEvent::HistoryRecordStored { id: chunk })
        );
    }

    #[test]
    fn test_usage_skips_non_history_keys() {
        let (state, mut rx) = testing::swarm_loop_state::<u32>().expect("swarm loop state");
        assert_eq!(
            testing::libp2p_event_from_kademlia_result(
                &state,
                &mut rx,
                testing::kademlia_record_found(kad::RecordKey::new(&"zone/zone/zone-a"))
            ),
            None
        );
    }
}
