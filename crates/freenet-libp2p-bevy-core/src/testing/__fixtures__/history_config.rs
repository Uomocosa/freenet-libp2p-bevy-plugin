use std::time::Duration;

use crate::p2p;
use crate::replica_sync;

#[must_use]
pub fn history_config() -> replica_sync::HistoryConfig {
    replica_sync::HistoryConfig {
        write_interval: Duration::from_millis(80),
        record_ttl: Duration::from_secs(90),
        per_zone_bytes: p2p::HistoryByteCount::from(64u64 * 1024 * 1024),
        max_fetch_retries: 2,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::history_config;

    #[test]
    fn test_usage() {
        let config = history_config();
        assert_eq!(config.write_interval, Duration::from_millis(80));
        assert_eq!(config.max_fetch_retries, 2);
    }
}
