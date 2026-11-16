use crate::testing;

pub fn commit_url(board: &testing::harness::GitHubCheckpointBoard) -> String {
    format!(
        "https://api.github.com/repos/{}/{}/commits/{}",
        board.owner, board.repo, board.sha
    )
}

#[cfg(test)]
mod tests {
    use super::commit_url;
    use crate::testing;

    #[test]
    fn test_usage() {
        let board = testing::harness::GitHubCheckpointBoard::new(
            "owner".to_string(),
            "repo".to_string(),
            "abc123".to_string(),
            "run-1".to_string(),
        );
        assert_eq!(
            commit_url(&board),
            "https://api.github.com/repos/owner/repo/commits/abc123"
        );
    }
}
