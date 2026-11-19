use crate::p2p;

#[must_use]
pub const fn is_ready_event<Payload>(event: &p2p::Event<Payload>) -> bool {
    matches!(event, p2p::Event::Libp2p(p2p::Libp2pEvent::Ready(_)))
}

#[cfg(test)]
mod tests {
    use crate::p2p;
    use crate::testing;

    use super::is_ready_event;

    #[test]
    fn test_usage() {
        let ready = p2p::Libp2pEvent::Ready(testing::peer("a", &[]));
        assert!(is_ready_event::<u8>(&p2p::Event::Libp2p(ready)));
        assert!(!is_ready_event::<u8>(&p2p::Event::Error("e".to_string())));
    }
}
