use crate::p2p;

pub fn remove_history_records<Payload: p2p::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    prefix: &p2p::HistoryKeyPrefix,
) {
    use libp2p::kad::store::RecordStore;
    let store = state.swarm.behaviour_mut().kademlia.store_mut();
    let doomed: Vec<_> = store
        .records()
        .filter(|record| {
            matches!(
                p2p::RoomHistoryRecordKind::parse_history_record_key(&record.key),
                Some(p2p::RoomHistoryRecordKind::Manifest(ref found)) if found == prefix
            ) || matches!(
                p2p::RoomHistoryRecordKind::parse_history_record_key(&record.key),
                Some(p2p::RoomHistoryRecordKind::Chunk(ref chunk)) if chunk.prefix == *prefix
            )
        })
        .map(|record| record.key.clone())
        .collect();
    for key in &doomed {
        store.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::remove_history_records;

    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, _rx) = p2p::SwarmLoopState::<u32>::for_test();
        let gone = p2p::HistoryKeyPrefix::from("gone");
        let kept = p2p::HistoryKeyPrefix::from("kept");
        for prefix in [&gone, &kept] {
            state.put_history_record(
                &p2p::RoomHistoryRecordKind::Manifest(prefix.clone()),
                vec![1],
                std::time::Duration::from_secs(90),
            );
        }
        remove_history_records(&mut state, &gone);
        let measured = state.measure_history_store();
        assert!(!measured.contains_key(&gone));
        assert!(measured.contains_key(&kept));
    }
}
