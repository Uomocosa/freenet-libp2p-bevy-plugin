use crate::p2p;

#[must_use]
pub fn libp2p_event<Payload>(event: p2p::Event<Payload>) -> Option<p2p::Libp2pEvent> {
    match event {
        p2p::Event::Libp2p(event) => Some(event),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::p2p;
    use crate::testing;

    use super::libp2p_event;

    #[test]
    fn test_usage() {
        let ready = p2p::Libp2pEvent::Ready(testing::peer("a", &[]));
        assert_eq!(
            libp2p_event::<u8>(p2p::Event::Libp2p(ready.clone())),
            Some(ready)
        );
        assert_eq!(libp2p_event::<u8>(p2p::Event::Error("e".to_string())), None);
    }
}
