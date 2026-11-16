use std::time::Duration;

pub const PEER_WINDOWS: [(i32, i32); 4] = [(0, 0), (481, 0), (0, 260), (481, 260)];

pub const POLL_INTERVAL: Duration = Duration::from_millis(250);

pub const FAKE_ACTION: Duration = Duration::from_secs(60);
pub const FAKE_LOBBY: Duration = Duration::from_secs(120);
pub const REAL_NETWORK_MULTIPLIER: f64 = 3.5;
pub const MILLIS_PER_DAY: u64 = 86_400_000;
pub const DISCOVER_DEADLINE: Duration = Duration::from_secs(180);
pub const EXIT_DEADLINE: Duration = Duration::from_secs(30);
pub const DEFAULT_RUST_LOG: &str = "info,starboard=debug,lobby_room=debug,discovery=debug";
pub const LOG_TAIL_LINES: usize = 60;
pub const NOISE_MARKER: &str = "history size against the per-process limit";

// no test_usage necessary
