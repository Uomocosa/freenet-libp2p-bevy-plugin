pub const LEASE_INTERVAL_PER_SPECTATOR: std::time::Duration = std::time::Duration::from_millis(500);
pub const LEASE_TTL_MULTIPLIER: u32 = 3;
pub const MAX_LEASE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(120);
pub const MAX_SLICE_REQUESTS_PER_PEER: usize = 4;
pub const MIN_LEASE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(15);
pub const SLICE_REQUEST_COOLDOWN: std::time::Duration = std::time::Duration::from_secs(5);

// no test_usage necessary
