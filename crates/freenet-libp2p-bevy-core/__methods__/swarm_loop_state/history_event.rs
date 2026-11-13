use crate::p2p;
use libp2p::kad;

pub fn history_event(result: kad::QueryResult) -> Option<p2p::Libp2pEvent> {
    match result {
        kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(peer_record))) => {
            let record = peer_record.record;
            Some(p2p::Libp2pEvent::HistoryRecordFound {
                id: p2p::RoomHistoryRecordKind::parse_history_record_key(&record.key)?,
                value: record.value,
            })
        }
        kad::QueryResult::GetRecord(Err(
            kad::GetRecordError::NotFound { key, .. } | kad::GetRecordError::Timeout { key },
        )) => Some(p2p::Libp2pEvent::HistoryRecordNotFound {
            id: p2p::RoomHistoryRecordKind::parse_history_record_key(&key)?,
        }),
        kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
            Some(p2p::Libp2pEvent::HistoryRecordStored {
                id: p2p::RoomHistoryRecordKind::parse_history_record_key(&key)?,
            })
        }
        kad::QueryResult::PutRecord(Err(error)) => Some(p2p::Libp2pEvent::HistoryRecordPutFailed {
            id: p2p::RoomHistoryRecordKind::parse_history_record_key(error.key())?,
            reason: error.to_string(),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use libp2p::kad;

    use super::history_event;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let key = kad::RecordKey::new(&"lobby/history/prefix/manifest");
        let result = kad::QueryResult::GetRecord(Err(kad::GetRecordError::NotFound {
            key,
            closest_peers: Vec::new(),
        }));
        assert!(matches!(
            history_event(result),
            Some(p2p::Libp2pEvent::HistoryRecordNotFound { .. })
        ));
    }
}
