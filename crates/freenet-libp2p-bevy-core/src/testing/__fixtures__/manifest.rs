use crate::p2p;

#[must_use]
pub fn manifest(prefix: &str) -> p2p::ZoneHistoryRecordKind {
    p2p::ZoneHistoryRecordKind::Manifest(p2p::HistoryKeyPrefix::from(prefix))
}

#[cfg(test)]
mod tests {
    use super::manifest;

    #[test]
    fn test_usage() {
        assert!(matches!(
            manifest("abc"),
            crate::p2p::ZoneHistoryRecordKind::Manifest(_)
        ));
    }
}
