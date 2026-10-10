use std::path::Path;
use std::process::{Command, Stdio};

use super::terminal_guard::TerminalGuard;

const DEFAULT_RUST_LOG: &str =
    "info,lobby_room=debug,p2p=debug,room_lobby=debug,freenet_libp2p_bevy_plugin=debug";

/// # Errors
/// Returns an error if the log cannot be created or the app fails to spawn.
pub fn spawn_app(
    bin: &Path,
    args: &[String],
    username: &str,
    log: &Path,
) -> Result<TerminalGuard, String> {
    let log_file =
        std::fs::File::create(log).map_err(|e| format!("create log {}: {e}", log.display()))?;
    let err_file = log_file
        .try_clone()
        .map_err(|e| format!("clone log {}: {e}", log.display()))?;
    let title = format!("lobby-{username}");
    let rust_log = std::env::var("LOBBY_RUST_LOG").unwrap_or_else(|_| DEFAULT_RUST_LOG.to_string());
    let child = Command::new(bin)
        .args(args)
        .env("RUST_LOG", rust_log)
        .stdin(Stdio::null())
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(err_file))
        .spawn()
        .map_err(|e| format!("spawn app failed: {e}"))?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    Ok(TerminalGuard {
        child: Some(child),
        window_title: title,
        log: log.to_path_buf(),
    })
}
