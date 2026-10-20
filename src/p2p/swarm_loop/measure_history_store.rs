use std::collections::BTreeMap;

use crate::p2p;

pub fn measure_history_store<Payload: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<Payload>>,
) -> BTreeMap<p2p::HistoryKeyPrefix, p2p::HistoryByteCount> {
    use libp2p::kad::store::RecordStore;
    swarm
        .behaviour_mut()
        .kademlia
        .store_mut()
        .records()
        .filter_map(|record| {
            let prefix = match p2p::parse_history_record_key(&record.key)? {
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
    use libp2p::identity::Keypair;

    use super::measure_history_store;
    use crate::identity;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let mut swarm = p2p::build_swarm::<u32>(
            Keypair::generate_ed25519(),
            p2p::MdnsMode::Disabled,
            &identity::GameId::from("test"),
            &p2p::GossipsubConfig::default(),
        )
        .unwrap();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let prefix = p2p::HistoryKeyPrefix::from("abc");
        let chunk = p2p::RoomHistoryRecordKind::Chunk(p2p::HistoryChunkId {
            prefix: prefix.clone(),
            content_hash: p2p::ContentHash::from("h"),
            chunk: p2p::ChunkIndex(0),
        });
        p2p::swarm_loop::put_history_record(
            &mut swarm,
            &tx,
            &p2p::RoomHistoryRecordKind::Manifest(prefix.clone()),
            vec![1, 2, 3],
        );
        p2p::swarm_loop::put_history_record(&mut swarm, &tx, &chunk, vec![0; 10]);
        let measured = measure_history_store(&mut swarm);
        assert_eq!(measured.get(&prefix), Some(&p2p::HistoryByteCount(13)));
    }
}
