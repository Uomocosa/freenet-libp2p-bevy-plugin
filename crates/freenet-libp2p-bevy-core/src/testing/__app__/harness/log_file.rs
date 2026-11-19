use std::path::PathBuf;

use atomic_delegate_macros::atomic_delegates;
use derive_more::{Deref, From};

#[derive(Debug, Clone, PartialEq, Eq, Deref, From)]
#[from(forward)]
pub struct LogFile(pub PathBuf);

impl LogFile {
    #[must_use]
    pub fn contains(&self, needle: &str) -> bool {
        let Self(path) = self;
        let text = std::fs::read_to_string(path).unwrap_or_default();
        Self::strip_ansi(&text).contains(needle)
    }
}

#[atomic_delegates]
impl LogFile {
    pub fn read_head(&self, max_bytes: u64) -> String {}
    pub fn marker_times(&self, needle: &str) -> Vec<(u64, String)> {}
}

#[atomic_delegates]
impl LogFile {
    pub(crate) fn strip_ansi(text: &str) -> String {}
    pub(crate) fn line_millis(line: &str) -> Option<u64> {}
}

#[cfg(test)]
mod tests {
    use super::LogFile;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("peer.log");
        std::fs::write(&path, "2026-10-01T17:04:19.867Z hello\n").expect("write log");
        let log = LogFile::from(path);
        assert!(log.contains("hello"));
        assert!(!log.contains("missing"));
        assert!(log.read_head(1024).contains("hello"));
        let times = log.marker_times("hello");
        assert_eq!(times.len(), 1);
    }
}
