use std::time::Duration;

pub const CONNECT_RETRY: Duration = Duration::from_secs(5);
pub const MISSING_RETRY: Duration = Duration::from_secs(2);
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);

// no test_usage necessary
