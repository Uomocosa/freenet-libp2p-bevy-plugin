use std::io::Write;
use std::process::Child;

use crate::testing;

pub fn stop_capture(mut process: Child) {
    if let Some(mut stdin) = process.stdin.take() {
        let _ = stdin.write_all(b"q");
        let _ = stdin.flush();
    }
    for _ in 0..testing::harness::FFMPEG_SHUTDOWN_ATTEMPTS {
        if !matches!(process.try_wait(), Ok(None)) {
            return;
        }
        std::thread::sleep(testing::harness::FFMPEG_SHUTDOWN_POLL);
    }
    let _ = process.kill();
    let _ = process.wait();
}

// no test_usage necessary
