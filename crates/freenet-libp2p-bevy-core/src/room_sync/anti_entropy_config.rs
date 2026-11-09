use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AntiEntropyConfig {
    pub interval: Duration,
    pub max_slice_fetches_per_peer: usize,
}

impl Default for AntiEntropyConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(2),
            max_slice_fetches_per_peer: 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::room_sync;

    #[test]
    fn test_usage() {
        let config = room_sync::AntiEntropyConfig::default();
        assert_eq!(config.interval, Duration::from_secs(2));
        assert_eq!(config.max_slice_fetches_per_peer, 8);
    }
}
