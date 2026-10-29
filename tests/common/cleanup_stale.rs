use std::process::Command;
use std::time::Duration;

pub fn cleanup_stale() {
    for example in ["examples/lobby_room", "examples/starboard"] {
        let _ = Command::new("pkill").args(["-f", example]).output();
    }
    std::thread::sleep(Duration::from_secs(2));
}
