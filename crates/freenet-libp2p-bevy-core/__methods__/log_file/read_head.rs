use std::io::Read;

use crate::testing;

pub fn read_head(log: &testing::harness::LogFile, max_bytes: u64) -> String {
    let testing::harness::LogFile(path) = log;
    let Ok(file) = std::fs::File::open(path) else {
        return String::new();
    };
    let mut buf = Vec::new();
    let _ = file.take(max_bytes).read_to_end(&mut buf);
    testing::harness::LogFile::strip_ansi(&String::from_utf8_lossy(&buf))
}

#[cfg(test)]
mod tests {
    use super::read_head;
    use crate::testing;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("peer.log");
        std::fs::write(&path, "2026-10-01T17:04:19.867Z hello\n").expect("write log");
        assert!(read_head(&testing::harness::LogFile::from(path), 1024).contains("hello"));
        let missing = testing::harness::LogFile::from(dir.path().join("missing.log"));
        assert_eq!(read_head(&missing, 1024), String::new());
    }
}
