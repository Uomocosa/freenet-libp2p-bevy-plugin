use std::process::Command;
use std::time::Duration;

pub fn cleanup_stale() {
    for binary in ["/lobby_room", "/starboard"] {
        let _ = Command::new("pkill").args(["-f", binary]).output();
    }
    std::thread::sleep(Duration::from_secs(2));
}
