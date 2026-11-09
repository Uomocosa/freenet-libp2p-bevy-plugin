/// The folder the embedded node uses for config, data and logs; Temporary is
/// removed on drop, Kept persists. Holding it keeps the temp dir alive while
/// the node runs.
pub enum FreenetNodeDir {
    Temporary(tempfile::TempDir),
    Kept(std::path::PathBuf),
}
