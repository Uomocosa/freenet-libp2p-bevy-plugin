use crate::p2p;
use libp2p::kad;
use std::time::Duration;

pub fn put_history_record<Payload: p2p::traits::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    id: &p2p::ZoneHistoryRecordKind,
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
            .send(p2p::Event::Libp2p(
                p2p::Libp2pEvent::HistoryRecordPutFailed {
                    id: id.clone(),
                    reason: format!("{error:?}"),
                },
            ))
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::put_history_record;

    use crate::p2p;
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, mut rx) = testing::swarm_loop_state::<u32>().expect("swarm loop state");
        put_history_record(
            &mut state,
            &testing::manifest("abc"),
            vec![1, 2, 3],
            Duration::from_secs(90),
        );
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_usage_reports_a_value_the_store_refuses() {
        let (mut state, mut rx) = testing::swarm_loop_state::<u32>().expect("swarm loop state");
        put_history_record(
            &mut state,
            &testing::manifest("abc"),
            vec![0; 70 * 1024],
            Duration::from_secs(90),
        );
        assert!(matches!(
            rx.try_recv(),
            Ok(p2p::Event::Libp2p(
                p2p::Libp2pEvent::HistoryRecordPutFailed { .. }
            ))
        ));
    }
}
