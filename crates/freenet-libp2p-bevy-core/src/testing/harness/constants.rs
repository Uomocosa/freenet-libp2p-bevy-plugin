use std::time::Duration;

pub const PEER_WINDOWS: [(i32, i32); 4] = [(0, 0), (481, 0), (0, 260), (481, 260)];

pub const E2E_DISPLAY_ENV: &str = "E2E_DISPLAY";
pub const E2E_XVFB_ENV: &str = "E2E_XVFB";
pub const E2E_RECORD_ENV: &str = "E2E_RECORD";
pub const E2E_RECORD_DRY_RUN_ENV: &str = "E2E_RECORD_DRY_RUN";
pub const E2E_FFMPEG_ENV: &str = "E2E_FFMPEG";

pub const XVFB_FIRST_DISPLAY: u16 = 90;
pub const XVFB_LAST_DISPLAY: u16 = 120;

/// A private X server must hold every `PEER_WINDOWS` tile of a 960x540 app window:
/// the last tile starts at (481, 260), so the bounding box is 1441x800 and the width is
/// rounded up to 1442 because yuv420p video needs even dimensions.
pub const XVFB_SCREEN_SIZE: &str = "1442x800x24";

pub const E2E_CAPTURE_SIZE: &str = "1442x800";
pub const E2E_RECORDING_FPS: &str = "10";
pub const E2E_RECORDING_CRF: &str = "30";
pub const E2E_RECORDING_RECODE_CRF: &str = "36";

/// Telegram caps a video at 50 MB; 45 MiB leaves room for the multipart envelope.
pub const TELEGRAM_VIDEO_BYTES: u64 = 47_185_920;

pub const XVFB_START_TIMEOUT: Duration = Duration::from_secs(5);
pub const XVFB_POLL_INTERVAL: Duration = Duration::from_millis(50);
pub const FFMPEG_SHUTDOWN_ATTEMPTS: u32 = 100;
pub const FFMPEG_SHUTDOWN_POLL: Duration = Duration::from_millis(100);

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
