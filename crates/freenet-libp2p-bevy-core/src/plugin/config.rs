use std::marker::PhantomData;

use crate::identity;
use crate::p2p;

pub struct Config<Payload: p2p::traits::Message> {
    pub mode: p2p::TransportMode,
    pub mdns: p2p::MdnsMode,
    pub game: identity::GameId,
    pub gossipsub: p2p::GossipsubConfig,
    pub keypair: Option<identity::SavedKeypair>,
    pub marker: PhantomData<Payload>,
}

impl<Payload: p2p::traits::Message> Config<Payload> {
    #[must_use]
    pub fn new(mode: p2p::TransportMode, mdns: p2p::MdnsMode) -> Self {
        Self {
            mode,
            mdns,
            game: identity::GameId::from("test"),
            gossipsub: p2p::GossipsubConfig::default(),
            keypair: None,
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

    #[must_use]
    pub fn with_keypair(mut self, keypair: identity::SavedKeypair) -> Self {
        self.keypair = Some(keypair);
        self
    }

    pub(crate) fn runner_keypair(&self) -> libp2p::identity::Keypair {
        let configured = self.keypair.as_ref().map(|saved| (**saved).clone());
        configured.unwrap_or_else(libp2p::identity::Keypair::generate_ed25519)
    }
}

#[cfg(test)]
mod tests {
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};

    use crate::identity;
    use crate::p2p;

    use super::Config;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let config = Config::<Dummy>::new(p2p::TransportMode::Both, p2p::MdnsMode::Disabled)
            .with_game(identity::GameId::from("test"));
        assert!(config.keypair.is_none());
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("id.identity");
        let saved = identity::SavedKeypair::try_from(path.as_path()).expect("saved");
        let expected = saved.public().to_peer_id();
        let configured = config.with_keypair(saved);
        assert_eq!(configured.runner_keypair().public().to_peer_id(), expected);
    }

    #[test]
    fn configs_without_keypair_produce_fresh_keys() {
        let first = Config::<Dummy>::new(p2p::TransportMode::Both, p2p::MdnsMode::Disabled)
            .runner_keypair();
        let second = Config::<Dummy>::new(p2p::TransportMode::Both, p2p::MdnsMode::Disabled)
            .runner_keypair();
        assert_ne!(first.public().to_peer_id(), second.public().to_peer_id());
    }
}
