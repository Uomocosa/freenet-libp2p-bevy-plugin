use std::process::{Command, Stdio};

use crate::testing;

pub fn start_capture(recording: &mut testing::harness::RunRecording, display: Option<&str>) {
    let Some(capture) = display.map(|name| testing::harness::ScreenCapture::from(name.to_string()))
    else {
        eprintln!(
            "e2e recording: {} not recorded, the run has no E2E display",
            recording.test_name
        );
        return;
    };
    let clip = recording.clip_path();
    let binary = testing::harness::RunRecording::ffmpeg_binary();
    let mut command = Command::new(&binary);
    command
        .args(capture.ffmpeg_arguments(&clip))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    capture.apply_environment(&mut command);
    match command.spawn() {
        Ok(child) => {
            recording.capture = Some(capture);
            recording.process = Some(child);
        }
        Err(error) => eprintln!("e2e recording: starting {binary}: {error}"),
    }
}

// no test_usage necessary
