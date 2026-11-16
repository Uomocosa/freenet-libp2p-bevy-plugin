use std::time::Duration;

pub const PEER_WINDOWS: [(i32, i32); 4] = [(0, 0), (481, 0), (0, 260), (481, 260)];

pub const E2E_DISPLAY_ENV: &str = "E2E_DISPLAY";
pub const E2E_XVFB_ENV: &str = "E2E_XVFB";

pub const XVFB_FIRST_DISPLAY: u16 = 90;
pub const XVFB_LAST_DISPLAY: u16 = 120;

/// A private X server must hold every `PEER_WINDOWS` tile of a 960x540 app window:
/// the last tile starts at (481, 260), so the bounding box is 1441x800 and the width is
/// rounded up to 1442 because yuv420p video needs even dimensions.
pub const XVFB_SCREEN_SIZE: &str = "1442x800x24";

pub const XVFB_START_TIMEOUT: Duration = Duration::from_secs(5);
pub const XVFB_POLL_INTERVAL: Duration = Duration::from_millis(50);

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
