pub const LEASE_INTERVAL_PER_SPECTATOR: std::time::Duration = std::time::Duration::from_millis(500);
pub const LEASE_TTL_MULTIPLIER: u32 = 3;
/// How many recently applied op metas a spectator remembers for duplicate suppression.
///
/// Ops are assumed idempotent; beyond this window the oldest meta is forgotten, so an
/// already-applied op may be applied again instead of being dropped.
pub const MAX_APPLIED_METAS: usize = 1024;
pub const MAX_LEASE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(120);
pub const MAX_SLICE_REQUESTS_PER_PEER: usize = 4;
pub const MIN_LEASE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(15);
pub const SLICE_REQUEST_COOLDOWN: std::time::Duration = std::time::Duration::from_secs(5);
pub const SLICE_REPAIR_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

// no test_usage necessary
