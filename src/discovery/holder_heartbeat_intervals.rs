use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HolderHeartbeatIntervals {
    pub interval_between_heartbeats: Duration,
    pub ttl: Duration,
}

impl Default for HolderHeartbeatIntervals {
    fn default() -> Self {
        Self {
            interval_between_heartbeats: Duration::from_secs(30),
            ttl: Duration::from_secs(90),
        }
    }
}

// no test_usage necessary
