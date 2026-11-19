use crate::testing;

pub fn local_board() -> Result<(tempfile::TempDir, testing::harness::LocalCheckpointBoard), String>
{
    let dir = tempfile::tempdir().map_err(|error| format!("temp dir: {error}"))?;
    let board = testing::harness::LocalCheckpointBoard::new(dir.path().to_path_buf());
    Ok((dir, board))
}

#[cfg(test)]
mod tests {
    use super::local_board;

    #[test]
    fn test_usage() {
        let (_dir, board) = local_board().expect("local board");
        assert!(board.exists());
    }
}
