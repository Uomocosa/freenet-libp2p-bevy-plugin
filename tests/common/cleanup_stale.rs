use std::process::Command;
use std::time::Duration;

pub fn cleanup_stale() {
    let _ = Command::new("pkill")
        .args(["-f", "examples/lobby_room"])
        .output();
    std::thread::sleep(Duration::from_secs(2));
}
