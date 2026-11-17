use std::collections::{HashMap, HashSet};

use libp2p::gossipsub;

use crate::identity;

pub struct ZoneSubscriptions {
    pub game: identity::GameId,
    pub by_topic: HashMap<gossipsub::TopicHash, identity::ZoneName>,
    pub joined: identity::Zone,
    pub watched: HashSet<identity::ZoneName>,
}

impl ZoneSubscriptions {
    pub fn new(game: identity::GameId) -> Self {
        Self {
            game,
            by_topic: HashMap::new(),
            joined: identity::Zone::Global,
            watched: HashSet::new(),
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
        if !self.watched.contains(&zone) {
            self.by_topic.remove(&topic.hash());
        }
    }
    pub fn watch(&mut self, topic: &gossipsub::IdentTopic, zone: identity::ZoneName) {
        self.by_topic.insert(topic.hash(), zone.clone());
        self.watched.insert(zone);
    }
    pub fn unwatch(&mut self, topic: &gossipsub::IdentTopic, zone: &identity::ZoneName) {
        self.watched.remove(zone);
        if self.joined.restricted() != Some(zone) {
            self.by_topic.remove(&topic.hash());
        }
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
