use std::path::Path;

/// The `--result-json` flag and its per-peer file, next to the peer's game log
/// under the run directory.
pub fn result_json_arg(run_dir: &Path, username: &str) -> [String; 2] {
    let file = run_dir.join(username).join("result.json");
    ["--result-json".to_string(), file.display().to_string()]
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::result_json_arg;

    #[test]
    fn test_usage() {
        let [flag, file] = result_json_arg(Path::new("/tmp/run"), "p1");
        assert_eq!(flag, "--result-json");
        assert!(file.ends_with("p1/result.json"));
    }
}
