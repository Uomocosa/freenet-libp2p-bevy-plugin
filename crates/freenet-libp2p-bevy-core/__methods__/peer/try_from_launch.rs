use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::testing;

pub async fn try_from_launch(
    bin: &Path,
    args: &[String],
    display: Option<&str>,
) -> Result<testing::harness::Peer, String> {
    let port = testing::harness::Peer::free_port()?;
    let log_dir = tempfile::tempdir().map_err(|e| format!("temp dir: {e}"))?;
    let log = log_dir.path().join("peer.log");
    let stdout = File::create(&log).map_err(|e| format!("create {}: {e}", log.display()))?;
    let stderr = stdout.try_clone().map_err(|e| format!("clone log: {e}"))?;
    let mut command = Command::new(bin);
    command
        .args(args)
        .arg("--e2e-port")
        .arg(port.to_string())
        .env(
            "RUST_LOG",
            std::env::var("E2E_RUST_LOG")
                .unwrap_or_else(|_| testing::harness::DEFAULT_RUST_LOG.to_string()),
        )
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr);
    if let Some(display) = display {
        command.env("DISPLAY", display).env_remove("XAUTHORITY");
    }
    if let Some(icd) = testing::harness::Peer::lavapipe_icd() {
        command.env("VK_ICD_FILENAMES", icd);
    }
    let child = command
        .spawn()
        .map_err(|e| format!("spawn {}: {e}", bin.display()))?;
    let mut peer = testing::harness::Peer {
        child,
        client: testing::harness::BrpClient::new(port),
        log,
        _log_dir: log_dir,
    };
    peer.await_brp().await?;
    Ok(peer)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::try_from_launch;

    #[tokio::test]
    async fn test_usage() {
        let missing = Path::new("/nonexistent-e2e-bin");
        assert!(try_from_launch(missing, &[], None).await.is_err());
    }
}
