use std::collections::BTreeMap;

use crate::identity;
use crate::p2p;
use crate::testing;

pub fn serve_history_command(
    router: &mut testing::PeerMessageRouter,
    from: &identity::PeerId,
    command: p2p::Libp2pCommand,
) -> Vec<testing::RoutedEvent> {
    match command {
        p2p::Libp2pCommand::PutHistoryRecord { id, value, .. } => {
            let key: Vec<u8> = id.history_record_key().as_ref().to_vec();
            router.history_values.insert(key.clone(), value);
            router.history_kinds.insert(key, id.clone());
            vec![testing::RoutedEvent {
                to: from.clone(),
                event: p2p::Libp2pEvent::HistoryRecordStored { id },
            }]
        }
        p2p::Libp2pCommand::GetHistoryRecord { id } => {
            let key: Vec<u8> = id.history_record_key().as_ref().to_vec();
            let event = router.history_values.get(&key).map_or_else(
                || p2p::Libp2pEvent::HistoryRecordNotFound { id: id.clone() },
                |value| p2p::Libp2pEvent::HistoryRecordFound {
                    id: id.clone(),
                    value: value.clone(),
                },
            );
            vec![testing::RoutedEvent {
                to: from.clone(),
                event,
            }]
        }
        p2p::Libp2pCommand::MeasureHistoryStore => {
            let mut bytes_by_prefix: BTreeMap<p2p::HistoryKeyPrefix, p2p::HistoryByteCount> =
                BTreeMap::new();
            for (key, value) in &router.history_values {
                let Some(kind) = router.history_kinds.get(key) else {
                    continue;
                };
                let bytes = u64::try_from(value.len()).unwrap_or(u64::MAX);
                let total: &mut p2p::HistoryByteCount = bytes_by_prefix
                    .entry(kind.record_prefix().clone())
                    .or_default();
                *total = p2p::HistoryByteCount::from(total.saturating_add(bytes));
            }
            vec![testing::RoutedEvent {
                to: from.clone(),
                event: p2p::Libp2pEvent::HistoryStoreMeasured { bytes_by_prefix },
            }]
        }
        p2p::Libp2pCommand::RemoveHistoryRecords { prefix } => {
            let stale: Vec<Vec<u8>> = router
                .history_kinds
                .iter()
                .filter(|(_, kind)| *kind.record_prefix() == prefix)
                .map(|(key, _)| key.clone())
                .collect();
            for key in stale {
                router.history_values.remove(&key);
                router.history_kinds.remove(&key);
            }
            Vec::new()
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::serve_history_command;
    use crate::identity;
    use crate::p2p;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let peer = identity::PeerId::from("a");
        let events =
            serve_history_command(&mut router, &peer, p2p::Libp2pCommand::MeasureHistoryStore);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].to, peer);
        assert!(matches!(
            events[0].event,
            p2p::Libp2pEvent::HistoryStoreMeasured { .. }
        ));
    }
}
