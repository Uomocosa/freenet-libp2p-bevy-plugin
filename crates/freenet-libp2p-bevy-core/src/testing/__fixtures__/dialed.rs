use crate::identity;
use crate::testing;

#[must_use]
pub fn dialed(
    router: &mut testing::PeerMessageRouter,
    from: &identity::PeerId,
    to: &str,
) -> Vec<testing::RoutedEvent> {
    let target = testing::registered_peer(router, to);
    testing::dial(router, from, &target)
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::testing;

    use super::dialed;

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let from = testing::registered_peer(&mut router, "a");
        assert_eq!(dialed(&mut router, &from, "b").len(), 4);
    }
}
