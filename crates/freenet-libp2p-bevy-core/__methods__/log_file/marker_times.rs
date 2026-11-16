use crate::testing;

pub fn marker_times(log: &testing::harness::LogFile, needle: &str) -> Vec<(u64, String)> {
    let testing::harness::LogFile(path) = log;
    testing::harness::LogFile::strip_ansi(&std::fs::read_to_string(path).unwrap_or_default())
        .lines()
        .filter(|line| line.contains(needle))
        .filter_map(|line| {
            testing::harness::LogFile::line_millis(line).map(|ms| (ms, line.to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::marker_times;
    use crate::testing;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("peer.log");
        std::fs::write(
            &path,
            "2026-10-01T17:04:19.867Z hello\n2026-10-01T17:04:20.100Z other\n",
        )
        .expect("write log");
        let times = marker_times(&testing::harness::LogFile::from(path), "hello");
        assert_eq!(times.len(), 1);
        assert_eq!(times[0].1, "2026-10-01T17:04:19.867Z hello");
    }
}
