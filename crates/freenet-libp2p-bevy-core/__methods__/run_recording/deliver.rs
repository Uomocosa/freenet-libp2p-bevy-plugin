use std::path::Path;
use std::time::Duration;

use telegram_bot_helper::{Caption, TelegramBot};

use crate::testing;

pub fn deliver(
    test_name: &str,
    outcome: testing::harness::RecordingOutcome,
    elapsed: Duration,
    clip: &Path,
) {
    let caption = testing::harness::RunRecording::caption(test_name, outcome, elapsed);
    if testing::harness::RunRecording::is_dry_run() {
        println!(
            "e2e recording: dry run, clip={} caption={caption}",
            clip.display()
        );
        return;
    }
    let path = clip.to_path_buf();
    let _ = std::thread::spawn(move || {
        TelegramBot::send_video_best_effort(&path, &Caption::from(caption.as_str()));
    })
    .join();
}

// no test_usage necessary
