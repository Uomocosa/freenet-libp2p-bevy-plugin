use crate::identity;
use crate::testing;

#[must_use]
pub fn is_subscribed(router: &testing::PeerMessageRouter, peer: &identity::PeerId) -> bool {
    router
        .subscribed_topics
        .get(peer)
        .is_some_and(|topics| !topics.is_empty())
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::testing;

    use super::is_subscribed;

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let peer = testing::registered_peer(&mut router, "a");
        assert!(!is_subscribed(&router, &peer));
        let _ = testing::subscribe(&mut router, &peer, "news");
        assert!(is_subscribed(&router, &peer));
    }
}
