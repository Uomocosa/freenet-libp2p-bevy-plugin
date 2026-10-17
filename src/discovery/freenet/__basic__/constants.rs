use std::time::Duration;

pub const CONNECT_RETRY: Duration = Duration::from_secs(5);
pub const MISSING_RETRY: Duration = Duration::from_secs(2);
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(75);
pub const DEPLOY_TIMEOUT: Duration = Duration::from_secs(30);
pub const REPUBLISH: Duration = Duration::from_secs(30);
pub const LOBBY_REFRESH: Duration = Duration::from_secs(5);
pub const RESUBSCRIBE: Duration = Duration::from_secs(10);
pub const RING_WAIT: Duration = Duration::from_secs(30);

// no test_usage necessary
