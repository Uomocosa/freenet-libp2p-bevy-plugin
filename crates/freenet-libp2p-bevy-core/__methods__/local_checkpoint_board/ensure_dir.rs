use crate::testing;

pub fn ensure_dir(board: &testing::harness::LocalCheckpointBoard) -> Result<(), String> {
    std::fs::create_dir_all(board.as_path())
        .map_err(|error| format!("board dir {}: {error}", board.display()))
}

#[cfg(test)]
mod tests {
    use super::ensure_dir;
    use crate::testing;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().expect("temp dir");
        let nested = dir.path().join("nested").join("board");
        let board = testing::harness::LocalCheckpointBoard::new(nested.clone());
        ensure_dir(&board).expect("ensure");
        assert!(nested.is_dir());
    }
}
