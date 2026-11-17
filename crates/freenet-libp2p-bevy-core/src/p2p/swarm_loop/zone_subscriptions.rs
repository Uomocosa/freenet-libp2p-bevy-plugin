use std::collections::HashMap;

use libp2p::gossipsub;

use crate::identity;

pub struct ZoneSubscriptions {
    pub game: identity::GameId,
    pub by_topic: HashMap<gossipsub::TopicHash, identity::ZoneName>,
    pub joined: identity::Zone,
    pub watched: HashMap<identity::ZoneName, u32>,
}

impl ZoneSubscriptions {
    pub fn new(game: identity::GameId) -> Self {
        Self {
            game,
            by_topic: HashMap::new(),
            joined: identity::Zone::Global,
            watched: HashMap::new(),
        }
    }
}

impl ZoneSubscriptions {
    pub fn register(&mut self, topic: &gossipsub::IdentTopic, zone: identity::ZoneName) {
        self.by_topic.insert(topic.hash(), zone.clone());
        self.joined = identity::Zone::Restricted(zone);
    }
    pub fn unregister(&mut self, topic: &gossipsub::IdentTopic) {
        let Some(zone) = self.by_topic.get(&topic.hash()).cloned() else {
            return;
        };
        if self.joined.restricted() == Some(&zone) {
            self.joined = identity::Zone::Global;
        }
        if !self.watched.contains_key(&zone) {
            self.by_topic.remove(&topic.hash());
        }
    }
    pub fn watch(&mut self, topic: &gossipsub::IdentTopic, zone: identity::ZoneName) {
        self.by_topic.insert(topic.hash(), zone.clone());
        let watchers = self.watched.entry(zone).or_insert(0);
        *watchers = watchers.saturating_add(1);
    }
    pub fn unwatch(&mut self, topic: &gossipsub::IdentTopic, zone: &identity::ZoneName) -> bool {
        let Some(remaining) = self.remaining_watchers_after_unwatch(zone) else {
            return false;
        };
        if remaining == 0 {
            self.release_unwatched_zone(topic, zone);
        }
        remaining == 0
    }
}

#[rustfmt::skip]
impl ZoneSubscriptions {
    pub fn zone(&self, topic: &gossipsub::TopicHash) -> Option<&identity::ZoneName> {
        self.by_topic.get(topic)
    }
    pub fn zone_topic(&self, zone: &identity::ZoneName) -> gossipsub::IdentTopic {
        gossipsub::IdentTopic::new(identity::Topic::from_zone(&self.game, zone).as_str())
    }
}

impl ZoneSubscriptions {
    pub(crate) fn remaining_watchers_after_unwatch(
        &mut self,
        zone: &identity::ZoneName,
    ) -> Option<u32> {
        let watchers = self.watched.get_mut(zone)?;
        *watchers = watchers.saturating_sub(1);
        Some(*watchers)
    }
    pub(crate) fn release_unwatched_zone(
        &mut self,
        topic: &gossipsub::IdentTopic,
        zone: &identity::ZoneName,
    ) {
        self.watched.remove(zone);
        if self.joined.restricted() != Some(zone) {
            self.by_topic.remove(&topic.hash());
        }
    }
}

#[cfg(test)]
mod tests {
    use libp2p::gossipsub;

    use super::ZoneSubscriptions;
    use crate::identity;

    #[test]
    fn test_usage() {
        let zone = identity::ZoneName::from("zone-a");
        let mut zones = ZoneSubscriptions::new(identity::GameId::from("game"));
        let topic =
            gossipsub::IdentTopic::new(identity::Topic::from_zone(&zones.game, &zone).as_str());
        assert_eq!(zones.zone(&topic.hash()), None);
        assert_eq!(zones.joined, identity::Zone::Global);
        zones.register(&topic, zone.clone());
        assert_eq!(zones.zone(&topic.hash()), Some(&zone));
        assert_eq!(zones.joined, identity::Zone::Restricted(zone));
        zones.unregister(&topic);
        assert_eq!(zones.zone(&topic.hash()), None);
        assert_eq!(zones.joined, identity::Zone::Global);
    }

    fn zone_topic(game: &identity::GameId, zone: &identity::ZoneName) -> gossipsub::IdentTopic {
        gossipsub::IdentTopic::new(identity::Topic::from_zone(game, zone).as_str())
    }

    #[test]
    fn watching_registers_the_topic_without_joining_the_zone() {
        let mut zones = ZoneSubscriptions::new(identity::GameId::from("game"));
        let zone = identity::ZoneName::from("zone-a");
        let topic = zone_topic(&zones.game, &zone);
        zones.watch(&topic, zone.clone());
        assert_eq!(zones.zone(&topic.hash()), Some(&zone));
        assert_eq!(zones.joined, identity::Zone::Global);
        zones.unwatch(&topic, &zone);
        assert_eq!(zones.zone(&topic.hash()), None);
        assert_eq!(zones.joined, identity::Zone::Global);
    }

    #[test]
    fn a_second_watch_keeps_the_zone_after_one_unwatch() {
        let mut zones = ZoneSubscriptions::new(identity::GameId::from("game"));
        let zone = identity::ZoneName::from("zone-a");
        let topic = zone_topic(&zones.game, &zone);
        zones.watch(&topic, zone.clone());
        zones.watch(&topic, zone.clone());
        assert!(!zones.unwatch(&topic, &zone));
        assert_eq!(zones.zone(&topic.hash()), Some(&zone));
        assert!(zones.unwatch(&topic, &zone));
        assert_eq!(zones.zone(&topic.hash()), None);
        assert!(!zones.watched.contains_key(&zone));
    }

    #[test]
    fn unwatching_an_unknown_zone_changes_nothing() {
        let mut zones = ZoneSubscriptions::new(identity::GameId::from("game"));
        let zone = identity::ZoneName::from("zone-a");
        let topic = zone_topic(&zones.game, &zone);
        assert!(!zones.unwatch(&topic, &zone));
        assert_eq!(zones.zone(&topic.hash()), None);
        assert!(zones.watched.is_empty());
    }

    #[test]
    fn a_second_watch_keeps_a_joined_zone_after_one_unwatch() {
        let mut zones = ZoneSubscriptions::new(identity::GameId::from("game"));
        let zone = identity::ZoneName::from("zone-a");
        let topic = zone_topic(&zones.game, &zone);
        zones.watch(&topic, zone.clone());
        zones.watch(&topic, zone.clone());
        zones.register(&topic, zone.clone());
        assert!(!zones.unwatch(&topic, &zone));
        assert_eq!(zones.zone(&topic.hash()), Some(&zone));
        assert!(zones.unwatch(&topic, &zone));
        assert_eq!(zones.zone(&topic.hash()), Some(&zone));
        assert_eq!(zones.joined, identity::Zone::Restricted(zone));
        zones.unregister(&topic);
        assert_eq!(zones.zone(&topic.hash()), None);
        assert_eq!(zones.joined, identity::Zone::Global);
    }

    #[test]
    fn unwatching_a_joined_zone_keeps_the_topic_and_the_join() {
        let mut zones = ZoneSubscriptions::new(identity::GameId::from("game"));
        let zone = identity::ZoneName::from("zone-a");
        let topic = zone_topic(&zones.game, &zone);
        zones.watch(&topic, zone.clone());
        zones.register(&topic, zone.clone());
        zones.unwatch(&topic, &zone);
        assert_eq!(zones.zone(&topic.hash()), Some(&zone));
        assert_eq!(zones.joined, identity::Zone::Restricted(zone));
    }

    #[test]
    fn leaving_a_watched_zone_keeps_the_watched_topic() {
        let mut zones = ZoneSubscriptions::new(identity::GameId::from("game"));
        let zone = identity::ZoneName::from("zone-a");
        let topic = zone_topic(&zones.game, &zone);
        zones.register(&topic, zone.clone());
        zones.watch(&topic, zone.clone());
        zones.unregister(&topic);
        assert_eq!(zones.zone(&topic.hash()), Some(&zone));
        assert_eq!(zones.joined, identity::Zone::Global);
        zones.unwatch(&topic, &zone);
        assert_eq!(zones.zone(&topic.hash()), None);
    }
}
