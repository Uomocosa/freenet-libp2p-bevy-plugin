use std::time::Duration;

/// Bounds silent `FetchOrDeploy` fetch time.
///
/// River paths are as of commit `b78f81f` of `github.com/freenet/river`.
/// Value from River `REPUT_DELAY_MS` = 20000 in
/// `ui/src/components/app/freenet_api/constants.rs`,
/// "Delay before re-PUTting a contract when subscription fails".
/// Needed because on a brand-new key the `NotFound` can fail to arrive.
/// Interleaved mainnet A/B, 8 runs each at N=3: 20s and 40s tied on passes,
/// deployers and room visibility.
pub const FETCH_SILENCE_LIMIT: Duration = Duration::from_secs(20);

/// Caps lobby fetch sends.
///
/// Sends at 0s and 10s. A third send at 20s would go out exactly
/// at the silence limit and could not be answered.
pub const FETCH_MAX_SENDS: usize = 2;

/// Bounds lobby fetch resend spacing.
///
/// No River equivalent. A `Get` goes to one peer and waits out
/// freenet's 60s `OPERATION_TTL`.
pub const FETCH_RESEND_AFTER: Duration = Duration::from_secs(10);

/// Bounds a lobby `Put`.
///
/// River paths are as of commit `b78f81f` of `github.com/freenet/river`.
/// Value from River `cli/src/api.rs` `PUT_RESPONSE_TIMEOUT` = 60s.
/// Evidence: at 30s a `Put` that landed was reported as timed out on
/// mainnet run `lobby_room_discovery-1793979938`.
pub const DEPLOY_TIMEOUT: Duration = Duration::from_secs(60);

/// Bounds a `FetchOnly` fetch.
///
/// River paths are as of commit `b78f81f` of `github.com/freenet/river`.
/// Stays above freenet's 60s `OPERATION_TTL`. Only matters for peers
/// that never deploy. River's rule: `INVITATION_TIMEOUT_MS`
/// "must be > Freenet's `OPERATION_TTL` (60s) to avoid premature retry".
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(75);

/// Bounds one freenet websocket connect.
///
/// River paths are as of commit `b78f81f` of `github.com/freenet/river`.
/// Moved from the hard-coded value in
/// `methods/client/try_from_host_and_port.rs`.
/// Value from River `CONNECTION_TIMEOUT_MS` = 5000.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Bounds first reconnect delay.
///
/// River paths are as of commit `b78f81f` of `github.com/freenet/river`.
/// Doubling with +-20% jitter up to `RECONNECT_MAX`. Values from River
/// `RECONNECT_INITIAL_MS`, `RECONNECT_MAX_MS` and `reconnect_delay_ms`
/// in `ui/src/components/app/freenet_api/freenet_synchronizer.rs`.
/// Used by `from_host_and_port_with_retry` and anything else that
/// used `CONNECT_RETRY`.
pub const RECONNECT_INITIAL: Duration = Duration::from_secs(3);

/// Bounds capped reconnect delay.
///
/// River paths are as of commit `b78f81f` of `github.com/freenet/river`.
/// Exponential backoff with +-20% jitter. Values from River
/// `RECONNECT_INITIAL_MS`, `RECONNECT_MAX_MS` and `reconnect_delay_ms`
/// in `ui/src/components/app/freenet_api/freenet_synchronizer.rs`.
/// Used by `from_host_and_port_with_retry` and anything else that
/// used `CONNECT_RETRY`.
pub const RECONNECT_MAX: Duration = Duration::from_secs(60);

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

/// Bounds query time.
///
/// Covers peer queries and the lobby refresh budget.
/// No external source.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);

/// Bounds idle retry wait.
///
/// Covers waiting after a `ContractNotFound` when idle.
/// No external source.
pub const MISSING_RETRY: Duration = Duration::from_secs(2);

/// Bounds lobby poll drain.
///
/// Covers draining lobby notifications in `poll`.
/// No external source.
pub const DRAIN_TIMEOUT: Duration = Duration::from_millis(10);

/// Bounds held lobby rooms.
///
/// Evicts the oldest beyond this.
/// No external source.
pub const MAX_LOBBY_ROOMS: usize = 256;

pub const LOBBY_CONTRACT_WASM: &[u8] =
    include_bytes!("../../../../contract/lobby/lobby_contract.wasm");

// no test_usage necessary
