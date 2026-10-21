use std::time::Duration;

use bevy::prelude::Resource;

use crate::p2p;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoryConfig {
    pub write_interval: Duration,
    pub record_ttl: Duration,
    pub per_room_bytes: p2p::HistoryByteCount,
    pub max_fetch_retries: u32,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            write_interval: Duration::from_secs(30),
            record_ttl: Duration::from_secs(90),
            per_room_bytes: p2p::HistoryByteCount::from(64u64 * 1024 * 1024),
            max_fetch_retries: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::p2p;
    use crate::room_sync;

    #[test]
    fn test_usage() {
        let config = room_sync::HistoryConfig::default();
        assert_eq!(config.write_interval, Duration::from_secs(30));
        assert_eq!(config.record_ttl, Duration::from_secs(90));
        assert_eq!(
            config.per_room_bytes,
            p2p::HistoryByteCount::from(64u64 * 1024 * 1024)
        );
        assert_eq!(config.max_fetch_retries, 3);
    }
}
