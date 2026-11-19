use crate::p2p;
use crate::replica_sync;

#[must_use]
pub fn history_record_not_found_input<Op>() -> replica_sync::state_machine::Input<Op> {
    replica_sync::state_machine::Input::HistoryRecordNotFound {
        id: p2p::ZoneHistoryRecordKind::Manifest(p2p::HistoryKeyPrefix::from("prefix")),
    }
}

#[cfg(test)]
mod tests {
    use crate::p2p;
    use crate::replica_sync;

    use super::history_record_not_found_input;

    #[test]
    fn test_usage() {
        let input = history_record_not_found_input::<u8>();
        let replica_sync::state_machine::Input::HistoryRecordNotFound { id } = input else {
            panic!("expected a missing history record input");
        };
        assert_eq!(
            id,
            p2p::ZoneHistoryRecordKind::Manifest(p2p::HistoryKeyPrefix::from("prefix"))
        );
    }
}
