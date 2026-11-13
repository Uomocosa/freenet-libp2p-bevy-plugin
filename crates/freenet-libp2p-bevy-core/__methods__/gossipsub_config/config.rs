use libp2p::gossipsub;

use crate::p2p;

pub fn config(
    settings: &p2p::GossipsubConfig,
) -> Result<gossipsub::Config, gossipsub::ConfigBuilderError> {
    gossipsub::ConfigBuilder::default()
        .heartbeat_interval(settings.heartbeat)
        .mesh_n_low(settings.mesh_n_low)
        .mesh_n(settings.mesh_n)
        .mesh_n_high(settings.mesh_n_high)
        .max_transmit_size(settings.max_transmit_size)
        .flood_publish(settings.is_flood_publish_enabled)
        .build()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::config;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let built = config(&p2p::GossipsubConfig::default()).expect("valid config");
        assert_eq!(built.heartbeat_interval(), Duration::from_millis(100));
        assert_eq!(built.mesh_n_low(), 2);
        assert_eq!(built.mesh_n(), 4);
        assert_eq!(built.mesh_n_high(), 8);
        assert_eq!(built.max_transmit_size(), 65536);
        assert!(built.flood_publish());
    }
}
