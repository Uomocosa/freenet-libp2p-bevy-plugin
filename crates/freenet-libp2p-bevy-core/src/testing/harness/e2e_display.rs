use std::path::{Path, PathBuf};
use std::process::Child;

use atomic_delegate_macros::atomic_delegates;

pub struct E2EDisplay {
    pub name: String,
    pub(crate) child: Option<Child>,
}

#[atomic_delegates]
impl E2EDisplay {
    pub fn from_env(run_dir: &Path) -> Result<Option<Self>, String> {}
    pub(crate) fn start_virtual_display(run_dir: &Path) -> Result<Option<Self>, String> {}
}

#[atomic_delegates]
impl Drop for E2EDisplay {
    fn drop(&mut self) {}
}

#[atomic_delegates]
impl E2EDisplay {
    pub fn reused_display_name() -> Option<String> {}
}

#[atomic_delegates]
impl E2EDisplay {
    pub(crate) fn find_xvfb_binary() -> Result<PathBuf, String> {}
    pub(crate) fn free_display_name() -> Result<String, String> {}
}

#[rustfmt::skip]
impl E2EDisplay {
    pub(crate) fn socket_path(name: &str) -> PathBuf { PathBuf::from(format!("/tmp/.X11-unix/X{}", name.trim_start_matches(':'))) }
    pub(crate) fn lock_path(name: &str) -> PathBuf { PathBuf::from(format!("/tmp/.X{}-lock", name.trim_start_matches(':'))) }
    pub(crate) fn does_display_socket_exist(name: &str) -> bool { Self::socket_path(name).exists() }
    pub(crate) fn is_display_taken(name: &str) -> bool { Self::socket_path(name).exists() || Self::lock_path(name).exists() }
}

#[cfg(test)]
mod tests {
    use super::E2EDisplay;

    #[test]
    fn test_usage() {
        let free = E2EDisplay::free_display_name().expect("a display number in range");
        assert!(free.starts_with(':'));
        let reused = E2EDisplay::reused_display_name();
        assert!(reused.as_ref().is_none_or(|name| name.starts_with(':')));
    }
}
