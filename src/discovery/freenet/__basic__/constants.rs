use std::time::Duration;

pub const CONNECT_RETRY: Duration = Duration::from_secs(5);
pub const MISSING_RETRY: Duration = Duration::from_secs(2);
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(20);
pub const MIN_RING_PEERS: usize = 3;
pub const DEPLOY_TIMEOUT: Duration = Duration::from_secs(30);

// no test_usage necessary
