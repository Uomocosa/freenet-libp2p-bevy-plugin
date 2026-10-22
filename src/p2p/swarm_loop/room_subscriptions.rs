use std::collections::HashMap;

use libp2p::gossipsub;

use crate::identity;

pub struct RoomSubscriptions {
    pub game: identity::GameId,
    pub by_topic: HashMap<gossipsub::TopicHash, identity::RoomName>,
    pub joined: Option<identity::RoomName>,
}

impl RoomSubscriptions {
    pub fn new(game: identity::GameId) -> Self {
        Self {
            game,
            by_topic: HashMap::new(),
            joined: None,
        }
    }
}

#[rustfmt::skip]
impl RoomSubscriptions {
    pub fn register(&mut self, topic: &gossipsub::IdentTopic, room: identity::RoomName) {
        self.by_topic.insert(topic.hash(), room.clone());
        self.joined = Some(room);
    }
    pub fn unregister(&mut self, topic: &gossipsub::IdentTopic) {
        let removed = self.by_topic.remove(&topic.hash());
        if removed.as_ref() == self.joined.as_ref() { self.joined = None; }
    }
    pub fn room(&self, topic: &gossipsub::TopicHash) -> Option<&identity::RoomName> { self.by_topic.get(topic) }
}

#[cfg(test)]
mod tests {
    use libp2p::gossipsub;

    use super::RoomSubscriptions;
    use crate::identity;

    #[test]
    fn test_usage() {
        let room = identity::RoomName::from("room-a");
        let mut rooms = RoomSubscriptions::new(identity::GameId::from("game"));
        let topic =
            gossipsub::IdentTopic::new(identity::Topic::from_room(&rooms.game, &room).as_str());
        assert_eq!(rooms.room(&topic.hash()), None);
        assert_eq!(rooms.joined, None);
        rooms.register(&topic, room.clone());
        assert_eq!(rooms.room(&topic.hash()), Some(&room));
        assert_eq!(rooms.joined, Some(room));
        rooms.unregister(&topic);
        assert_eq!(rooms.room(&topic.hash()), None);
        assert_eq!(rooms.joined, None);
    }
}
