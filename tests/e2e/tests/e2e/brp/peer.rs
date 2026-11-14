use std::fs::{self, File};
use std::net::{Ipv4Addr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use serde_json::Value;
use tempfile::TempDir;

use super::BrpClient;
use super::poll_until::POLL_INTERVAL;

const DISCOVER_DEADLINE: Duration = Duration::from_secs(180);
const EXIT_DEADLINE: Duration = Duration::from_secs(30);
const DEFAULT_RUST_LOG: &str = "info,starboard=debug,lobby_room=debug,discovery=debug";
const LOG_TAIL_LINES: usize = 60;
const NOISE_MARKER: &str = "history size against the per-process limit";

pub struct Peer {
    child: Child,
    client: BrpClient,
    log: PathBuf,
    _log_dir: TempDir,
}

impl Peer {
    /// # Errors
    /// Returns an error if the process cannot start or does not answer `rpc.discover`.
    pub async fn launch(bin: &Path, args: &[String]) -> Result<Self, String> {
        let port = free_port()?;
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
                std::env::var("E2E_RUST_LOG").unwrap_or_else(|_| DEFAULT_RUST_LOG.to_string()),
            )
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr);
        if let Some(icd) = lavapipe_icd() {
            command.env("VK_ICD_FILENAMES", icd);
        }
        let child = command
            .spawn()
            .map_err(|e| format!("spawn {}: {e}", bin.display()))?;
        let mut peer = Self {
            child,
            client: BrpClient::new(port),
            log,
            _log_dir: log_dir,
        };
        peer.await_brp().await?;
        Ok(peer)
    }

    /// # Errors
    /// Returns an error if the request fails or the result does not decode as `T`.
    pub async fn call<T: DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, String> {
        self.client.call(method, params).await
    }

    /// # Errors
    /// Returns an error if the process does not exit in time or exits unsuccessfully.
    pub async fn exit(&mut self) -> Result<(), String> {
        let _ = self.client.call::<Value>("e2e/exit", Value::Null).await;
        let started = Instant::now();
        loop {
            if let Some(status) = self.exit_status()? {
                return if status.success() {
                    Ok(())
                } else {
                    Err(self.explain(&format!("app exited with {status}")))
                };
            }
            if started.elapsed() >= EXIT_DEADLINE {
                return Err(self.explain("app did not exit after e2e/exit"));
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    /// # Errors
    /// Returns an error if the process cannot be killed or reaped.
    pub fn kill(&mut self) -> Result<(), String> {
        self.child
            .kill()
            .map_err(|e| self.explain(&format!("killing the app: {e}")))?;
        self.child
            .wait()
            .map(|_| ())
            .map_err(|e| self.explain(&format!("reaping the killed app: {e}")))
    }

    async fn await_brp(&mut self) -> Result<(), String> {
        let started = Instant::now();
        loop {
            if self
                .client
                .call::<Value>("rpc.discover", Value::Null)
                .await
                .is_ok()
            {
                return Ok(());
            }
            if let Some(status) = self.exit_status()? {
                return Err(self.explain(&format!(
                    "app exited before answering rpc.discover: {status}"
                )));
            }
            if started.elapsed() >= DISCOVER_DEADLINE {
                return Err(self.explain("rpc.discover did not answer"));
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    fn exit_status(&mut self) -> Result<Option<ExitStatus>, String> {
        self.child
            .try_wait()
            .map_err(|e| self.explain(&format!("checking the app process: {e}")))
    }

    /// The app's whole log so far. The file lives in the peer's temp dir, so it stays readable
    /// after the app exits until this `Peer` is dropped.
    #[must_use]
    pub fn log_text(&self) -> String {
        fs::read_to_string(&self.log).unwrap_or_default()
    }

    #[must_use]
    pub fn log_contains(&self, line: &str) -> bool {
        self.log_text().contains(line)
    }

    #[must_use]
    pub fn explain(&self, message: &str) -> String {
        let log = fs::read_to_string(&self.log).unwrap_or_default();
        let lines: Vec<&str> = log
            .lines()
            .filter(|line| !line.contains(NOISE_MARKER))
            .collect();
        let tail = lines
            .iter()
            .skip(lines.len().saturating_sub(LOG_TAIL_LINES))
            .copied()
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "{message}\n--- log tail ({}) ---\n{tail}",
            self.log.display()
        )
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn free_port() -> Result<u16, String> {
    let listener =
        TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).map_err(|e| format!("free port: {e}"))?;
    listener
        .local_addr()
        .map(|addr| addr.port())
        .map_err(|e| format!("free port: {e}"))
}

fn lavapipe_icd() -> Option<PathBuf> {
    let mut icds: Vec<PathBuf> = fs::read_dir("/usr/share/vulkan/icd.d")
        .ok()?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("lvp_icd"))
                && path
                    .extension()
                    .is_some_and(|extension| extension == "json")
        })
        .collect();
    icds.sort();
    icds.into_iter().next()
}
