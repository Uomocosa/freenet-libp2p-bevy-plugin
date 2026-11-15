use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HolderHeartbeatIntervals {
    pub interval_between_heartbeats: Duration,
    pub ttl: Duration,
    pub remote_clock_skew_margin: Duration,
}

impl Default for HolderHeartbeatIntervals {
    fn default() -> Self {
        Self {
            interval_between_heartbeats: Duration::from_secs(30),
            ttl: Duration::from_secs(90),
            remote_clock_skew_margin: Duration::from_secs(600),
        }
    }
}

// no test_usage necessary
