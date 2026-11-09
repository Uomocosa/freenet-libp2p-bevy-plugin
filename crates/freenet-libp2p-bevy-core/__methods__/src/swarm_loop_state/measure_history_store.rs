use std::collections::BTreeMap;

use crate::p2p;

pub fn measure_history_store<Payload: p2p::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
) -> BTreeMap<p2p::HistoryKeyPrefix, p2p::HistoryByteCount> {
    use libp2p::kad::store::RecordStore;
    state
        .swarm
        .behaviour_mut()
        .kademlia
        .store_mut()
        .records()
        .filter_map(|record| {
            let prefix = match p2p::RoomHistoryRecordKind::parse_history_record_key(&record.key)? {
                p2p::RoomHistoryRecordKind::Manifest(prefix) => prefix,
                p2p::RoomHistoryRecordKind::Chunk(chunk) => chunk.prefix,
            };
            Some((
                prefix,
                u64::try_from(record.value.len()).unwrap_or(u64::MAX),
            ))
        })
        .fold(BTreeMap::new(), |mut totals, (prefix, bytes)| {
            let total: &mut p2p::HistoryByteCount = totals.entry(prefix).or_default();
            *total = p2p::HistoryByteCount::from(total.saturating_add(bytes));
            totals
        })
}

#[cfg(test)]
mod tests {
    use super::measure_history_store;

    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, _rx) = p2p::SwarmLoopState::<u32>::for_test();
        let prefix = p2p::HistoryKeyPrefix::from("abc");
        let chunk = p2p::RoomHistoryRecordKind::Chunk(p2p::HistoryChunkId {
            prefix: prefix.clone(),
            content_hash: p2p::ContentHash::from("h"),
            chunk: p2p::ChunkIndex(0),
        });
        state.put_history_record(
            &p2p::RoomHistoryRecordKind::Manifest(prefix.clone()),
            vec![1, 2, 3],
            std::time::Duration::from_secs(90),
        );
        state.put_history_record(&chunk, vec![0; 10], std::time::Duration::from_secs(90));
        let measured = measure_history_store(&mut state);
        assert_eq!(measured.get(&prefix), Some(&p2p::HistoryByteCount(13)));
    }
}
