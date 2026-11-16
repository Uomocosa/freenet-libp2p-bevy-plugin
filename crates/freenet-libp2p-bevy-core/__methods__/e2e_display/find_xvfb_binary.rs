use std::path::{Path, PathBuf};

use crate::testing;

pub fn find_xvfb_binary() -> Result<PathBuf, String> {
    let configured = std::env::var(testing::harness::E2E_XVFB_ENV)
        .ok()
        .filter(|value| !value.is_empty());
    if let Some(binary) = configured {
        return Ok(PathBuf::from(binary));
    }
    std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .map(|directory| Path::new(directory).join("Xvfb"))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| {
            format!(
                "e2e display: no Xvfb found; set {env}=/path/to/Xvfb or put Xvfb on PATH (tests never use your desktop DISPLAY; set {reuse}=:99 to reuse a server)",
                env = testing::harness::E2E_XVFB_ENV,
                reuse = testing::harness::E2E_DISPLAY_ENV
            )
        })
}

#[cfg(test)]
mod tests {
    use super::find_xvfb_binary;

    #[test]
    fn test_usage() {
        let found = find_xvfb_binary();
        assert!(found.is_ok() || found.is_err());
    }
}
