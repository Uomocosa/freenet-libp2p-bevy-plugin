use libp2p::kad;

use crate::p2p;

#[must_use]
pub fn libp2p_event_from_kademlia_result<Payload: p2p::traits::Message>(
    state: &p2p::SwarmLoopState<Payload>,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<p2p::Event<Payload>>,
    result: kad::QueryResult,
) -> Option<p2p::Libp2pEvent> {
    state.handle_kademlia_query_result(result);
    match rx.try_recv() {
        Ok(p2p::Event::Libp2p(event)) => Some(event),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use libp2p::kad;

    use crate::p2p;
    use crate::testing;

    use super::libp2p_event_from_kademlia_result;

    #[test]
    fn test_usage() {
        let (state, mut rx) = testing::swarm_loop_state::<u32>().expect("swarm loop state");
        let chunk = testing::history_chunk("abc", "def", 5);
        assert_eq!(
            libp2p_event_from_kademlia_result(
                &state,
                &mut rx,
                testing::kademlia_record_found(chunk.history_record_key())
            ),
            Some(p2p::Libp2pEvent::HistoryRecordFound {
                id: chunk,
                value: vec![1, 2, 3],
            })
        );
        assert_eq!(
            libp2p_event_from_kademlia_result(
                &state,
                &mut rx,
                testing::kademlia_record_found(kad::RecordKey::new(&"global/topic"))
            ),
            None
        );
    }
}
