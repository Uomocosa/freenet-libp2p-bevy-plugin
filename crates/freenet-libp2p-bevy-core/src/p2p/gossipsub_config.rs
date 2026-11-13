use std::time::Duration;

use atomic_delegate_macros::atomic_delegates;
use libp2p::gossipsub;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GossipsubConfig {
    pub heartbeat: Duration,
    pub mesh_n_low: usize,
    pub mesh_n: usize,
    pub mesh_n_high: usize,
    pub max_transmit_size: usize,
    pub is_flood_publish_enabled: bool,
}

impl Default for GossipsubConfig {
    fn default() -> Self {
        Self {
            heartbeat: Duration::from_millis(100),
            mesh_n_low: 2,
            mesh_n: 4,
            mesh_n_high: 8,
            max_transmit_size: 65536,
            is_flood_publish_enabled: true,
        }
    }
}

#[atomic_delegates]
impl GossipsubConfig {
    pub fn config(&self) -> Result<gossipsub::Config, gossipsub::ConfigBuilderError> {}
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::GossipsubConfig;

    #[test]
    fn test_usage() {
        let config = GossipsubConfig::default();
        assert_eq!(config.heartbeat, Duration::from_millis(100));
        assert_eq!(config.max_transmit_size, 65536);
        assert!(config.is_flood_publish_enabled);
        let built = config.config().unwrap();
        assert_eq!(built.heartbeat_interval(), Duration::from_millis(100));
        assert_eq!(built.mesh_n_low(), 2);
        assert_eq!(built.mesh_n(), 4);
        assert_eq!(built.mesh_n_high(), 8);
        assert_eq!(built.max_transmit_size(), 65536);
        assert!(built.flood_publish());
    }
}
