use libp2p::kad;

use crate::p2p;

pub fn put_history_record<Payload: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<Payload>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
    id: &p2p::RoomHistoryRecordKind,
    value: Vec<u8>,
) {
    let record = kad::Record {
        key: p2p::history_record_key(id),
        value,
        publisher: None,
        expires: None,
    };
    if let Err(error) = swarm
        .behaviour_mut()
        .kademlia
        .put_record(record, kad::Quorum::One)
    {
        event_tx
            .send(p2p::Event::Net(p2p::NetEvent::HistoryRecordPutFailed {
                id: id.clone(),
                reason: format!("{error:?}"),
            }))
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use libp2p::identity::Keypair;

    use super::put_history_record;
    use crate::identity;
    use crate::p2p;

    fn swarm() -> libp2p::Swarm<p2p::Behaviour<u32>> {
        p2p::build_swarm::<u32>(
            Keypair::generate_ed25519(),
            p2p::MdnsMode::Disabled,
            &identity::GameId::from("test"),
            &p2p::GossipsubConfig::default(),
        )
        .unwrap()
    }

    fn manifest() -> p2p::RoomHistoryRecordKind {
        p2p::RoomHistoryRecordKind::Manifest(p2p::HistoryKeyPrefix::from("abc"))
    }

    #[tokio::test]
    async fn test_usage() {
        let mut swarm = swarm();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        put_history_record(&mut swarm, &tx, &manifest(), vec![1, 2, 3]);
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_usage_reports_a_value_the_store_refuses() {
        let mut swarm = swarm();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        put_history_record(&mut swarm, &tx, &manifest(), vec![0; 70 * 1024]);
        assert!(matches!(
            rx.try_recv(),
            Ok(p2p::Event::Net(
                p2p::NetEvent::HistoryRecordPutFailed { .. }
            ))
        ));
    }
}
