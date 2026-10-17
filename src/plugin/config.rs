use std::marker::PhantomData;

use crate::identity;
use crate::p2p;

pub struct Config<T: p2p::Message> {
    pub mode: p2p::TransportMode,
    pub mdns: p2p::MdnsMode,
    pub game: identity::GameId,
    pub gossipsub: p2p::GossipsubConfig,
    pub marker: PhantomData<T>,
}

impl<T: p2p::Message> Config<T> {
    #[must_use]
    pub fn new(mode: p2p::TransportMode, mdns: p2p::MdnsMode) -> Self {
        Self {
            mode,
            mdns,
            game: identity::GameId::from("test"),
            gossipsub: p2p::GossipsubConfig::default(),
            marker: PhantomData,
        }
    }

    #[must_use]
    pub fn with_game(mut self, game: identity::GameId) -> Self {
        self.game = game;
        self
    }

    #[must_use]
    pub const fn with_gossipsub(mut self, gossipsub: p2p::GossipsubConfig) -> Self {
        self.gossipsub = gossipsub;
        self
    }
}
// no test_usage necessary
