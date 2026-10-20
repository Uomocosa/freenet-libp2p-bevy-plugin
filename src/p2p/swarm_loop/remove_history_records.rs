use crate::p2p;

pub fn remove_history_records<Payload: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<Payload>>,
    prefix: &p2p::HistoryKeyPrefix,
) {
    use libp2p::kad::store::RecordStore;
    let store = swarm.behaviour_mut().kademlia.store_mut();
    let doomed: Vec<_> = store
        .records()
        .filter(|record| {
            matches!(
                p2p::parse_history_record_key(&record.key),
                Some(p2p::RoomHistoryRecordKind::Manifest(ref found)) if found == prefix
            ) || matches!(
                p2p::parse_history_record_key(&record.key),
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
    use libp2p::identity::Keypair;

    use super::remove_history_records;
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
        let gone = p2p::HistoryKeyPrefix::from("gone");
        let kept = p2p::HistoryKeyPrefix::from("kept");
        for prefix in [&gone, &kept] {
            p2p::swarm_loop::put_history_record(
                &mut swarm,
                &tx,
                &p2p::RoomHistoryRecordKind::Manifest(prefix.clone()),
                vec![1],
            );
        }
        remove_history_records(&mut swarm, &gone);
        let measured = p2p::swarm_loop::measure_history_store(&mut swarm);
        assert!(!measured.contains_key(&gone));
        assert!(measured.contains_key(&kept));
    }
}
