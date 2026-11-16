use std::path::PathBuf;

pub fn board_dir_from_env() -> PathBuf {
    let raw = std::env::var("E2E_BOARD_DIR").unwrap_or_default();
    let trimmed = raw.trim();
    if !trimmed.is_empty() {
        return PathBuf::from(trimmed);
    }
    std::env::temp_dir().join(format!("e2e-board-{}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::board_dir_from_env;

    #[test]
    fn test_usage() {
        unsafe {
            std::env::set_var("E2E_BOARD_DIR", "/tmp/e2e-board-test");
        }
        assert_eq!(board_dir_from_env(), PathBuf::from("/tmp/e2e-board-test"));
        unsafe {
            std::env::remove_var("E2E_BOARD_DIR");
        }
        assert!(board_dir_from_env().is_absolute());
    }
}
