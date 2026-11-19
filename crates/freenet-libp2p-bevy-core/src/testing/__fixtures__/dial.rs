use crate::identity;
use crate::p2p;
use crate::testing;

#[must_use]
pub fn dial(
    router: &mut testing::PeerMessageRouter,
    from: &identity::PeerId,
    to: &identity::PeerId,
) -> Vec<testing::RoutedEvent> {
    router.route(
        from,
        p2p::Libp2pCommand::Dial(testing::peer(to.as_str(), &[])),
    )
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::testing;

    use super::dial;

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let from = testing::registered_peer(&mut router, "a");
        let to = testing::registered_peer(&mut router, "b");
        assert_eq!(dial(&mut router, &from, &to).len(), 4);
    }
}
