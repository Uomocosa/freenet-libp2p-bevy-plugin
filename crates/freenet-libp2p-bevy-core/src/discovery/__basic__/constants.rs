use std::time::Duration;

pub const TICK: Duration = Duration::from_secs(1);

/// Bounds gateway connection wait.
///
/// No River equivalent: River only checks the socket; we wait because
/// freenet rejects a subscribing `Get` with `PeerNotJoined` before the
/// gateway handshake.
pub const GATEWAY_WAIT: Duration = Duration::from_secs(30);

/// Bounds replica-healing resubscribe spacing.
///
/// No River equivalent, River has no timer. Heals split replica groups
/// with a routed `Subscribe` carrying a summary. The freenet skill
/// suggests about 30s.
pub const RESUBSCRIBE: Duration = Duration::from_secs(10);

/// Bounds lobby refresh spacing.
///
/// Covers `fetch_and_subscribe` in the sync loop.
/// No external source.
pub const LOBBY_REFRESH: Duration = Duration::from_secs(5);

/// Bounds idle retry wait.
///
/// Covers waiting after a `ContractNotFound` when idle.
/// No external source.
pub const MISSING_RETRY: Duration = Duration::from_secs(2);

/// Bounds held lobby rooms.
///
/// Evicts the oldest beyond this.
/// No external source.
pub const MAX_LOBBY_ROOMS: usize = 256;

pub const LOBBY_CONTRACT_WASM: &[u8] =
    include_bytes!("../../../../lobby_contract/lobby_contract.wasm");

// no test_usage necessary
