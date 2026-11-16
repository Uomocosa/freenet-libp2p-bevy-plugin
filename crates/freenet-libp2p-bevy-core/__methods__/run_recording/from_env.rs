use std::path::Path;
use std::time::Instant;

use crate::testing;

pub fn from_env(
    run_dir: &Path,
    test_name: &str,
    display: Option<&str>,
) -> testing::harness::RunRecording {
    let mut recording = testing::harness::RunRecording {
        run_dir: run_dir.to_path_buf(),
        test_name: test_name.to_string(),
        started: Instant::now(),
        outcome: testing::harness::RecordingOutcome::default(),
        capture: None,
        process: None,
    };
    if testing::harness::RunRecording::is_record_requested() {
        testing::harness::RunRecording::start_capture(&mut recording, display);
    }
    recording
}

// no test_usage necessary
