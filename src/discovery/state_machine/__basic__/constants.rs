use std::time::Duration;

pub const HEARTBEAT_TTL: Duration = Duration::from_secs(120);
pub const MEMBER_GRACE: Duration = Duration::from_secs(10);
pub const REDIAL: Duration = Duration::from_secs(2);
pub const HANDSHAKE_EVERY: Duration = Duration::from_secs(5);
pub const JOIN_HEAL_TIMEOUT: Duration = Duration::from_secs(30);

// no test_usage necessary
