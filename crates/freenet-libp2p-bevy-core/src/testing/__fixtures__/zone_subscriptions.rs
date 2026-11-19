use crate::identity;
use crate::p2p;

#[must_use]
pub fn zone_subscriptions(game: &str) -> p2p::swarm_loop::ZoneSubscriptions {
    p2p::swarm_loop::ZoneSubscriptions::new(identity::GameId::from(game))
}

#[cfg(test)]
mod tests {
    use crate::identity;

    use super::zone_subscriptions;

    #[test]
    fn test_usage() {
        let subscriptions = zone_subscriptions("game");
        assert_eq!(subscriptions.game, identity::GameId::from("game"));
        assert_eq!(subscriptions.joined, identity::Zone::Global);
        assert!(subscriptions.watched.is_empty());
        assert!(subscriptions.by_topic.is_empty());
    }
}
