use std::time::Duration;

pub const CONNECT_RETRY: Duration = Duration::from_secs(5);
pub const MISSING_RETRY: Duration = Duration::from_secs(2);
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);
pub const DRAIN_TIMEOUT: Duration = Duration::from_millis(10);
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(75);
pub const FETCH_RESEND_AFTER: Duration = Duration::from_secs(10);
pub const FETCH_MAX_SENDS: usize = 3;
pub const FETCH_SILENCE_LIMIT: Duration = Duration::from_secs(40);
pub const DEPLOY_TIMEOUT: Duration = Duration::from_secs(30);
pub const LOBBY_REFRESH: Duration = Duration::from_secs(5);
pub const RESUBSCRIBE: Duration = Duration::from_secs(10);
pub const RING_WAIT: Duration = Duration::from_secs(30);
pub const MAX_LOBBY_ROOMS: usize = 256;
pub const LOBBY_CONTRACT_WASM: &[u8] =
    include_bytes!("../../../../contract/lobby/lobby_contract.wasm");

// no test_usage necessary
