use std::process::Command;

use crate::testing;

pub fn apply_environment(capture: &testing::harness::ScreenCapture, command: &mut Command) {
    command
        .env("DISPLAY", capture.as_str())
        .env_remove("XAUTHORITY");
}

// no test_usage necessary
