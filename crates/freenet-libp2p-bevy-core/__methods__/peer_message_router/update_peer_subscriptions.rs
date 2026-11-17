use crate::identity;
use crate::p2p;
use crate::testing;

pub fn update_peer_subscriptions(
    router: &mut testing::PeerMessageRouter,
    from: &identity::PeerId,
    command: p2p::Libp2pCommand,
) -> Vec<testing::RoutedEvent> {
    match command {
        p2p::Libp2pCommand::Subscribe { topic } => {
            router
                .subscribed_topics
                .entry(from.clone())
                .or_default()
                .insert(topic);
        }
        p2p::Libp2pCommand::Unsubscribe { topic } => {
            if let Some(topics) = router.subscribed_topics.get_mut(from) {
                topics.remove(&topic);
            }
        }
        p2p::Libp2pCommand::SubscribeZone { zone } => {
            let topic = identity::Topic::from_zone(&router.game, &zone);
            router
                .subscribed_topics
                .entry(from.clone())
                .or_default()
                .insert(topic);
            router.joined_zones.insert(from.clone(), zone);
        }
        p2p::Libp2pCommand::UnsubscribeZone { zone } => {
            let topic = identity::Topic::from_zone(&router.game, &zone);
            let is_watched = router
                .watched_zones
                .get(from)
                .is_some_and(|zones| zones.contains(&zone));
            if !is_watched && let Some(topics) = router.subscribed_topics.get_mut(from) {
                topics.remove(&topic);
            }
            if router.joined_zones.get(from) == Some(&zone) {
                router.joined_zones.remove(from);
            }
        }
        p2p::Libp2pCommand::WatchZone { zone } => {
            let topic = identity::Topic::from_zone(&router.game, &zone);
            router
                .subscribed_topics
                .entry(from.clone())
                .or_default()
                .insert(topic);
            router
                .watched_zones
                .entry(from.clone())
                .or_default()
                .insert(zone);
        }
        p2p::Libp2pCommand::UnwatchZone { zone } => {
            let topic = identity::Topic::from_zone(&router.game, &zone);
            if let Some(zones) = router.watched_zones.get_mut(from) {
                zones.remove(&zone);
            }
            let is_joined = router.joined_zones.get(from) == Some(&zone);
            if !is_joined && let Some(topics) = router.subscribed_topics.get_mut(from) {
                topics.remove(&topic);
            }
        }
        _ => {}
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::update_peer_subscriptions;
    use crate::identity;
    use crate::p2p;
    use crate::testing;

    fn registered(router: &mut testing::PeerMessageRouter, id: &str) -> identity::PeerId {
        let peer = identity::Peer {
            id: identity::PeerId::from(id),
            addrs: Vec::new(),
        };
        router.register_peer(&peer);
        peer.id
    }

    fn subscribed(router: &testing::PeerMessageRouter, id: &identity::PeerId) -> bool {
        router
            .subscribed_topics
            .get(id)
            .is_some_and(|topics| !topics.is_empty())
    }

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let peer = registered(&mut router, "a");
        let topic = identity::Topic::from("some/topic");
        let events = update_peer_subscriptions(
            &mut router,
            &peer,
            p2p::Libp2pCommand::Subscribe {
                topic: topic.clone(),
            },
        );
        assert_eq!(events, Vec::new());
        assert!(subscribed(&router, &peer));
        let events = update_peer_subscriptions(
            &mut router,
            &peer,
            p2p::Libp2pCommand::Unsubscribe { topic },
        );
        assert_eq!(events, Vec::new());
        assert!(!subscribed(&router, &peer));
    }

    #[test]
    fn zone_subscribe_tracks_the_zone_topic_and_the_joined_zone() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let peer = registered(&mut router, "a");
        let zone = identity::ZoneName::from("zone-a");
        let events = update_peer_subscriptions(
            &mut router,
            &peer,
            p2p::Libp2pCommand::SubscribeZone { zone: zone.clone() },
        );
        assert_eq!(events, Vec::new());
        let topic = identity::Topic::from_zone(&identity::GameId::from("game"), &zone);
        assert!(
            router
                .subscribed_topics
                .get(&peer)
                .is_some_and(|topics| topics.contains(&topic))
        );
        assert_eq!(router.joined_zones.get(&peer), Some(&zone));
        let events = update_peer_subscriptions(
            &mut router,
            &peer,
            p2p::Libp2pCommand::UnsubscribeZone { zone },
        );
        assert_eq!(events, Vec::new());
        assert!(!subscribed(&router, &peer));
        assert!(!router.joined_zones.contains_key(&peer));
    }

    #[test]
    fn watching_tracks_the_zone_topic_without_joining() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let peer = registered(&mut router, "a");
        let zone = identity::ZoneName::from("zone-a");
        update_peer_subscriptions(
            &mut router,
            &peer,
            p2p::Libp2pCommand::WatchZone { zone: zone.clone() },
        );
        let topic = identity::Topic::from_zone(&identity::GameId::from("game"), &zone);
        assert!(
            router
                .subscribed_topics
                .get(&peer)
                .is_some_and(|topics| topics.contains(&topic))
        );
        assert!(!router.joined_zones.contains_key(&peer));
        update_peer_subscriptions(&mut router, &peer, p2p::Libp2pCommand::UnwatchZone { zone });
        assert!(!subscribed(&router, &peer));
    }

    #[test]
    fn unwatching_a_joined_zone_keeps_the_topic_and_the_join() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let peer = registered(&mut router, "a");
        let zone = identity::ZoneName::from("zone-a");
        update_peer_subscriptions(
            &mut router,
            &peer,
            p2p::Libp2pCommand::SubscribeZone { zone: zone.clone() },
        );
        update_peer_subscriptions(
            &mut router,
            &peer,
            p2p::Libp2pCommand::WatchZone { zone: zone.clone() },
        );
        update_peer_subscriptions(
            &mut router,
            &peer,
            p2p::Libp2pCommand::UnwatchZone { zone: zone.clone() },
        );
        assert!(subscribed(&router, &peer));
        assert_eq!(router.joined_zones.get(&peer), Some(&zone));
    }

    #[test]
    fn leaving_a_watched_zone_keeps_the_topic() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let peer = registered(&mut router, "a");
        let zone = identity::ZoneName::from("zone-a");
        update_peer_subscriptions(
            &mut router,
            &peer,
            p2p::Libp2pCommand::WatchZone { zone: zone.clone() },
        );
        update_peer_subscriptions(
            &mut router,
            &peer,
            p2p::Libp2pCommand::SubscribeZone { zone: zone.clone() },
        );
        update_peer_subscriptions(
            &mut router,
            &peer,
            p2p::Libp2pCommand::UnsubscribeZone { zone: zone.clone() },
        );
        assert!(subscribed(&router, &peer));
        assert!(!router.joined_zones.contains_key(&peer));
        update_peer_subscriptions(&mut router, &peer, p2p::Libp2pCommand::UnwatchZone { zone });
        assert!(!subscribed(&router, &peer));
    }

    #[test]
    fn leaving_another_zone_keeps_the_joined_zone() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let peer = registered(&mut router, "a");
        let zone = identity::ZoneName::from("zone-a");
        update_peer_subscriptions(
            &mut router,
            &peer,
            p2p::Libp2pCommand::SubscribeZone { zone: zone.clone() },
        );
        update_peer_subscriptions(
            &mut router,
            &peer,
            p2p::Libp2pCommand::UnsubscribeZone {
                zone: identity::ZoneName::from("zone-b"),
            },
        );
        assert_eq!(router.joined_zones.get(&peer), Some(&zone));
    }
}
