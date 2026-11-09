use std::time::Duration;

pub const FRIEND_INITIAL_BACKOFF: Duration = Duration::from_secs(2);
pub const FRIEND_MAX_BACKOFF: Duration = Duration::from_secs(60);
pub const KNOWN_INITIAL_BACKOFF: Duration = Duration::from_secs(2);
pub const KNOWN_MAX_BACKOFF: Duration = Duration::from_secs(30);
pub const KNOWN_MAX_FAILURES: u32 = 3;
pub const TICK_INTERVAL: Duration = Duration::from_millis(250);

// no test_usage necessary
