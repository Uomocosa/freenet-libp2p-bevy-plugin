use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::Instant;

use atomic_delegate_macros::atomic_delegates;

use crate::testing;

pub struct RunRecording {
    pub(crate) run_dir: PathBuf,
    pub(crate) test_name: String,
    pub(crate) started: Instant,
    pub(crate) outcome: testing::harness::RecordingOutcome,
    pub(crate) capture: Option<testing::harness::ScreenCapture>,
    pub(crate) process: Option<Child>,
}

#[atomic_delegates]
impl RunRecording {
    pub fn from_env(run_dir: &Path, test_name: &str, display: Option<&str>) -> Self {}
}

#[atomic_delegates]
impl Drop for RunRecording {
    fn drop(&mut self) {}
}

#[rustfmt::skip]
impl RunRecording {
    pub const fn mark_passed(&mut self) { self.outcome = testing::harness::RecordingOutcome::Passed; }
    #[must_use]
    pub fn clip_path(&self) -> PathBuf { self.run_dir.join(format!("e2e-{}.mp4", self.test_name)) }
    #[must_use]
    pub fn is_record_requested() -> bool { std::env::var(testing::harness::E2E_RECORD_ENV).is_ok_and(|value| value == "1") }
    #[must_use]
    pub fn is_dry_run() -> bool { std::env::var(testing::harness::E2E_RECORD_DRY_RUN_ENV).is_ok_and(|value| value == "1") }
    pub(crate) fn ffmpeg_binary() -> String { std::env::var(testing::harness::E2E_FFMPEG_ENV).unwrap_or_else(|_| "ffmpeg".to_string()) }
}

#[atomic_delegates]
impl RunRecording {
    pub(crate) fn start_capture(recording: &mut RunRecording, display: Option<&str>) {}
    pub(crate) fn stop_capture(process: Child) {}
    pub(crate) fn fit_under_budget(clip: &Path) -> Result<PathBuf, String> {}
    pub(crate) fn deliver(
        test_name: &str,
        outcome: testing::harness::RecordingOutcome,
        elapsed: std::time::Duration,
        clip: &Path,
    ) {
    }
    pub(crate) fn caption(
        test_name: &str,
        outcome: testing::harness::RecordingOutcome,
        elapsed: std::time::Duration,
    ) -> String {
    }
}

#[cfg(test)]
mod tests {
    use super::RunRecording;
    use crate::testing;

    #[test]
    fn test_usage() {
        let inert = RunRecording::from_env(std::path::Path::new("/tmp/e2e-run"), "proof", None);
        assert_eq!(inert.outcome, testing::harness::RecordingOutcome::Failed);
        assert!(inert.clip_path().ends_with("e2e-proof.mp4"));
        assert!(inert.process.is_none());
    }
}
