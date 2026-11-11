use std::process::Command;
use std::time::Duration;

const STALE_GAME_BINARIES: [&str; 2] = ["lobby_room", "starboard"];

pub fn cleanup_stale() {
    for binary in STALE_GAME_BINARIES {
        let _ = pkill_exact_name(binary).output();
    }
    std::thread::sleep(Duration::from_secs(2));
}

fn pkill_exact_name(binary: &str) -> Command {
    let mut pkill = Command::new("pkill");
    pkill.arg("-x").arg(binary);
    pkill
}

#[cfg(all(test, unix))]
mod tests {
    use super::{STALE_GAME_BINARIES, pkill_exact_name};
    use std::os::unix::process::CommandExt;
    use std::process::{Command, Stdio};
    use std::time::Duration;

    #[test]
    fn stale_game_cleanup_leaves_the_test_binary_alive() -> Result<(), String> {
        let self_path = std::env::current_exe().map_err(|error| format!("current_exe: {error}"))?;
        let mut child = Command::new("/bin/sleep")
            .arg0(self_path.clone())
            .arg("2")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("spawn: {error}"))?;
        std::thread::sleep(Duration::from_millis(500));
        for pattern in STALE_GAME_BINARIES {
            let _ = pkill_exact_name(pattern)
                .output()
                .map_err(|error| format!("pkill {pattern}: {error}"))?;
        }
        std::thread::sleep(Duration::from_millis(300));
        let status = child.wait().map_err(|error| format!("wait: {error}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "cleanup patterns {STALE_GAME_BINARIES:?} killed a process running the test binary {self_path:?}: {status:?}"
            ))
        }
    }
}
