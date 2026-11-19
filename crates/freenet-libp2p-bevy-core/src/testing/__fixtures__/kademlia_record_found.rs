use libp2p::kad;

#[must_use]
pub fn kademlia_record_found(key: kad::RecordKey) -> kad::QueryResult {
    kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(kad::PeerRecord {
        peer: None,
        record: kad::Record {
            key,
            value: vec![1, 2, 3],
            publisher: None,
            expires: None,
        },
    })))
}

#[cfg(test)]
mod tests {
    use libp2p::kad;

    use super::kademlia_record_found;

    #[test]
    fn test_usage() {
        let key = kad::RecordKey::new(&"abc");
        let kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(record))) =
            kademlia_record_found(key.clone())
        else {
            panic!("expected a found record");
        };
        assert_eq!(record.record.key, key);
        assert_eq!(record.record.value, vec![1, 2, 3]);
    }
}
