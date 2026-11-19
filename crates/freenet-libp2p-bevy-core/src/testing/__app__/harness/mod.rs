mod brp_client;
pub use brp_client::BrpClient;

mod checkpoint_board;
pub(crate) use checkpoint_board::{BoardStatus, CachedStatuses};
pub use checkpoint_board::{GitHubCheckpointBoard, LocalCheckpointBoard};

mod constants;
pub(crate) use constants::{
    DEFAULT_RUST_LOG, DISCOVER_DEADLINE, E2E_CAPTURE_SIZE, E2E_DISPLAY_ENV, E2E_FFMPEG_ENV,
    E2E_RECORD_DRY_RUN_ENV, E2E_RECORD_ENV, E2E_RECORDING_CRF, E2E_RECORDING_FPS,
    E2E_RECORDING_RECODE_CRF, E2E_XVFB_ENV, EXIT_DEADLINE, FAKE_ACTION, FAKE_LOBBY,
    FFMPEG_SHUTDOWN_ATTEMPTS, FFMPEG_SHUTDOWN_POLL, LOG_TAIL_LINES, MILLIS_PER_DAY, NOISE_MARKER,
    REAL_NETWORK_MULTIPLIER, TELEGRAM_VIDEO_BYTES, XVFB_FIRST_DISPLAY, XVFB_LAST_DISPLAY,
    XVFB_POLL_INTERVAL, XVFB_SCREEN_SIZE, XVFB_START_TIMEOUT,
};
pub use constants::{PEER_WINDOWS, POLL_INTERVAL};

mod deadlines;
pub use deadlines::Deadlines;

mod day_millis;
pub use day_millis::DayMillis;

mod e2e_display;
pub use e2e_display::E2EDisplay;

mod keys_builder;
pub use keys_builder::KeysBuilder;

mod log_file;
pub use log_file::LogFile;

mod peer;
pub use peer::Peer;

mod polling;
pub use polling::Polling;

mod recording_outcome;
pub use recording_outcome::RecordingOutcome;

mod run_recording;
pub use run_recording::RunRecording;

mod screen_capture;
pub use screen_capture::ScreenCapture;

mod unix_nanos;
pub use unix_nanos::UnixNanos;

mod window_origin;
pub use window_origin::WindowOrigin;
