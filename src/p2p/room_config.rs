use crate::p2p;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoomConfig {
    pub gossipsub: p2p::GossipsubConfig,
    pub anti_entropy: Option<p2p::AntiEntropyConfig>,
}

impl Default for RoomConfig {
    fn default() -> Self {
        Self {
            gossipsub: p2p::GossipsubConfig::default(),
            anti_entropy: Some(p2p::AntiEntropyConfig::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RoomConfig;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let room = RoomConfig::default();
        assert_eq!(room.gossipsub, p2p::GossipsubConfig::default());
        assert_eq!(room.anti_entropy, Some(p2p::AntiEntropyConfig::default()));
    }
}
