use std::path::Path;

/// The `--freenet-node-dir` flag and its per-peer directory, next to the peer's
/// game log under the run directory.
pub fn freenet_node_dir_arg(run_dir: &Path, username: &str) -> [String; 2] {
    let dir = run_dir.join(format!("{username}-freenet"));
    ["--freenet-node-dir".to_string(), dir.display().to_string()]
}
