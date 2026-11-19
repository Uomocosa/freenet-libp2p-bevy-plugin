use crate::identity;
use crate::p2p;
use crate::testing;

#[must_use]
pub fn subscribe(
    router: &mut testing::PeerMessageRouter,
    peer: &identity::PeerId,
    topic: &str,
) -> Vec<testing::RoutedEvent> {
    router.route(
        peer,
        p2p::Libp2pCommand::Subscribe {
            topic: identity::Topic::from(topic),
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::testing;

    use super::subscribe;

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let peer = testing::registered_peer(&mut router, "a");
        assert_eq!(
            subscribe(&mut router, &peer, "news"),
            Vec::<testing::RoutedEvent>::new()
        );
    }
}
