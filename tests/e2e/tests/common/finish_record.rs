use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Child;

#[must_use]
pub fn finish_record(mut child: Child, path: &Path) -> Option<PathBuf> {
    if let Some(stdin) = child.stdin.as_mut() {
        let _ = stdin.write_all(b"q\n");
        let _ = stdin.flush();
    }
    let _ = child.wait();
    if path.exists() {
        Some(path.to_path_buf())
    } else {
        None
    }
}
