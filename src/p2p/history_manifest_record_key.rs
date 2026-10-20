use libp2p::kad::RecordKey;

use crate::p2p;

#[must_use]
pub fn history_manifest_record_key(prefix: &p2p::HistoryKeyPrefix) -> RecordKey {
    RecordKey::new(&format!("lobby/history/{}/manifest", prefix.as_str()))
}

#[cfg(test)]
mod tests {
    use super::history_manifest_record_key;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let key = history_manifest_record_key(&p2p::HistoryKeyPrefix::from("abc"));
        assert_ne!(key.as_ref().len(), 0);
    }
}
