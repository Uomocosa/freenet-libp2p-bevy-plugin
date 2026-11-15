use std::collections::HashMap;

use libp2p::gossipsub;

use crate::identity;

pub struct ZoneSubscriptions {
    pub game: identity::GameId,
    pub by_topic: HashMap<gossipsub::TopicHash, identity::ZoneName>,
    pub joined: Option<identity::ZoneName>,
}

impl ZoneSubscriptions {
    pub fn new(game: identity::GameId) -> Self {
        Self {
            game,
            by_topic: HashMap::new(),
            joined: None,
        }
    }
}

impl ZoneSubscriptions {
    pub fn register(&mut self, topic: &gossipsub::IdentTopic, zone: identity::ZoneName) {
        self.by_topic.insert(topic.hash(), zone.clone());
        self.joined = Some(zone);
    }
    pub fn unregister(&mut self, topic: &gossipsub::IdentTopic) {
        let removed = self.by_topic.remove(&topic.hash());
        if removed.as_ref() == self.joined.as_ref() {
            self.joined = None;
        }
    }
}

#[rustfmt::skip]
impl ZoneSubscriptions {
    pub fn zone(&self, topic: &gossipsub::TopicHash) -> Option<&identity::ZoneName> {
        self.by_topic.get(topic)
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
        assert_eq!(zones.joined, None);
        zones.register(&topic, zone.clone());
        assert_eq!(zones.zone(&topic.hash()), Some(&zone));
        assert_eq!(zones.joined, Some(zone));
        zones.unregister(&topic);
        assert_eq!(zones.zone(&topic.hash()), None);
        assert_eq!(zones.joined, None);
    }
}
