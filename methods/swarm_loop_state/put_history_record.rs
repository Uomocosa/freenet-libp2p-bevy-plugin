use std::time::Duration;

use libp2p::kad;

use crate::p2p;

pub fn put_history_record<Payload: p2p::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    id: &p2p::RoomHistoryRecordKind,
    value: Vec<u8>,
    ttl: Duration,
) {
    let record = kad::Record {
        key: id.history_record_key(),
        value,
        publisher: None,
        expires: std::time::Instant::now().checked_add(ttl),
    };
    if let Err(error) = state
        .swarm
        .behaviour_mut()
        .kademlia
        .put_record(record, kad::Quorum::One)
    {
        state
            .event_tx
            .send(p2p::Event::Net(p2p::NetEvent::HistoryRecordPutFailed {
                id: id.clone(),
                reason: format!("{error:?}"),
            }))
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::put_history_record;

    use crate::p2p;

    fn manifest() -> p2p::RoomHistoryRecordKind {
        p2p::RoomHistoryRecordKind::Manifest(p2p::HistoryKeyPrefix::from("abc"))
    }

    #[tokio::test]
    async fn test_usage() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        put_history_record(
            &mut state,
            &manifest(),
            vec![1, 2, 3],
            Duration::from_secs(90),
        );
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_usage_reports_a_value_the_store_refuses() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        put_history_record(
            &mut state,
            &manifest(),
            vec![0; 70 * 1024],
            Duration::from_secs(90),
        );
        assert!(matches!(
            rx.try_recv(),
            Ok(p2p::Event::Net(
                p2p::NetEvent::HistoryRecordPutFailed { .. }
            ))
        ));
    }
}
