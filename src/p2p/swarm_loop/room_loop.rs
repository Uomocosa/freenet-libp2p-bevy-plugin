use std::collections::HashMap;

use crate::identity;
use crate::p2p;

pub struct RoomLoop {
    pub queries: HashMap<libp2p::kad::QueryId, identity::RoomName>,
    pub topics: p2p::swarm_loop::RoomTopics,
    pub pending: HashMap<u64, libp2p::request_response::ResponseChannel<p2p::RoomSyncFrame>>,
    pub next_request: u64,
    pub config: p2p::RoomConfig,
}

impl RoomLoop {
    pub fn new(game: identity::GameId, config: p2p::RoomConfig) -> Self {
        Self {
            queries: HashMap::new(),
            topics: p2p::swarm_loop::RoomTopics::new(game),
            pending: HashMap::new(),
            next_request: 0,
            config,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RoomLoop;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let rooms = RoomLoop::new(identity::GameId::from("game"), p2p::RoomConfig::default());
        assert!(rooms.queries.is_empty());
        assert_eq!(rooms.topics.game, identity::GameId::from("game"));
    }
}
