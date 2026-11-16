use std::net::{Ipv4Addr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, ExitStatus};

use atomic_delegate_macros::atomic_delegates;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::testing;

pub struct Peer {
    pub(crate) child: Child,
    pub(crate) client: testing::harness::BrpClient,
    pub log: PathBuf,
    pub(crate) _log_dir: tempfile::TempDir,
}

#[atomic_delegates]
impl Peer {
    pub async fn try_from_launch(bin: &Path, args: &[String]) -> Result<Self, String> {}
}

#[atomic_delegates]
impl Drop for Peer {
    fn drop(&mut self) {}
}

#[atomic_delegates]
impl Peer {
    pub async fn exit(&mut self) -> Result<(), String> {}
    #[must_use]
    pub fn explain(&self, message: &str) -> String {}
}

#[rustfmt::skip]
impl Peer {
    pub async fn call<T: DeserializeOwned>(&self, method: &str, params: Value) -> Result<T, String> { self.client.call(method, params).await }
    #[must_use]
    pub fn log_text(&self) -> String { std::fs::read_to_string(&self.log).unwrap_or_default() }
    #[must_use]
    pub fn log_contains(&self, line: &str) -> bool { self.log_text().contains(line) }
}

impl Peer {
    pub fn kill(&mut self) -> Result<(), String> {
        self.child
            .kill()
            .map_err(|e| self.explain(&format!("killing the app: {e}")))?;
        self.child
            .wait()
            .map(|_| ())
            .map_err(|e| self.explain(&format!("reaping the killed app: {e}")))
    }
    pub(crate) fn free_port() -> Result<u16, String> {
        let listener =
            TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).map_err(|e| format!("free port: {e}"))?;
        listener
            .local_addr()
            .map(|addr| addr.port())
            .map_err(|e| format!("free port: {e}"))
    }
    pub(crate) fn lavapipe_icd() -> Option<PathBuf> {
        let mut icds: Vec<PathBuf> = std::fs::read_dir("/usr/share/vulkan/icd.d")
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
}

#[atomic_delegates]
impl Peer {
    pub(crate) async fn await_brp(&mut self) -> Result<(), String> {}
    pub(crate) fn exit_status(&mut self) -> Result<Option<ExitStatus>, String> {}
}

#[cfg(test)]
mod tests {
    use super::Peer;

    #[test]
    fn test_usage() {
        assert!(Peer::free_port().is_ok());
        let _ = Peer::lavapipe_icd();
    }
}
