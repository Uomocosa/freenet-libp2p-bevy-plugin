use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

use crate::testing;

pub fn start_virtual_display(
    run_dir: &Path,
) -> Result<Option<testing::harness::E2EDisplay>, String> {
    if cfg!(not(target_os = "linux")) {
        return Ok(None);
    }
    let binary = testing::harness::E2EDisplay::find_xvfb_binary()?;
    let name = testing::harness::E2EDisplay::free_display_name()?;
    let log_path = run_dir.join("xvfb.log");
    let log = std::fs::File::create(&log_path)
        .map_err(|error| format!("xvfb log {}: {error}", log_path.display()))?;
    let stderr = log
        .try_clone()
        .map_err(|error| format!("xvfb log clone: {error}"))?;
    let mut child = Command::new(&binary)
        .arg(&name)
        .args([
            "-screen",
            "0",
            testing::harness::XVFB_SCREEN_SIZE,
            "-nolisten",
            "tcp",
        ])
        .stdin(Stdio::null())
        .stdout(log)
        .stderr(stderr)
        .spawn()
        .map_err(|error| format!("starting {} on {name}: {error}", binary.display()))?;
    let started = Instant::now();
    while started.elapsed() < testing::harness::XVFB_START_TIMEOUT {
        if testing::harness::E2EDisplay::does_display_socket_exist(&name) {
            return Ok(Some(testing::harness::E2EDisplay {
                name,
                child: Some(child),
            }));
        }
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!(
                "{name} exited while starting: {status}; see {}",
                log_path.display()
            ));
        }
        std::thread::sleep(testing::harness::XVFB_POLL_INTERVAL);
    }
    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_file(testing::harness::E2EDisplay::socket_path(&name));
    let _ = std::fs::remove_file(testing::harness::E2EDisplay::lock_path(&name));
    Err(format!(
        "{name} did not start within {:?}; see {}",
        testing::harness::XVFB_START_TIMEOUT,
        log_path.display()
    ))
}

// no test_usage necessary
