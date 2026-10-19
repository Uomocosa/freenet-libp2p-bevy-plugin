use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaseTimings {
    pub renew_every: Duration,
    pub ttl: Duration,
}

impl Default for LeaseTimings {
    fn default() -> Self {
        Self {
            renew_every: Duration::from_secs(30),
            ttl: Duration::from_secs(90),
        }
    }
}

// no test_usage necessary
