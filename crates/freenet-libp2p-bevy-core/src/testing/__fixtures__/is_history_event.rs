use crate::p2p;

#[must_use]
pub const fn is_history_event<Payload>(event: &p2p::Event<Payload>) -> bool {
    matches!(
        event,
        p2p::Event::Libp2p(
            p2p::Libp2pEvent::HistoryRecordStored { .. }
                | p2p::Libp2pEvent::HistoryRecordPutFailed { .. }
                | p2p::Libp2pEvent::HistoryRecordFound { .. }
                | p2p::Libp2pEvent::HistoryRecordNotFound { .. }
                | p2p::Libp2pEvent::HistoryStoreMeasured { .. }
        )
    )
}

#[cfg(test)]
mod tests {
    use crate::p2p;
    use crate::testing;

    use super::is_history_event;

    #[test]
    fn test_usage() {
        assert!(is_history_event::<u8>(&p2p::Event::Libp2p(
            p2p::Libp2pEvent::HistoryRecordStored {
                id: testing::manifest("abc")
            }
        )));
        assert!(!is_history_event::<u8>(&p2p::Event::Error("e".to_string())));
    }
}
