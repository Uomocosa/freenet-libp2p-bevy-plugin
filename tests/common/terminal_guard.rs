use std::path::PathBuf;
use std::process::Child;

pub struct TerminalGuard {
    pub child: Option<Child>,
    pub window_title: String,
    pub log: PathBuf,
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
