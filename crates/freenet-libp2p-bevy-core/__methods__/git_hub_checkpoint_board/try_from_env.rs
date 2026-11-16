use crate::testing;

pub fn try_from_env() -> Result<testing::harness::GitHubCheckpointBoard, String> {
    let repository = std::env::var("GITHUB_REPOSITORY").unwrap_or_default();
    let (owner, repo) = repository
        .split_once('/')
        .filter(|(owner, repo)| !owner.trim().is_empty() && !repo.trim().is_empty())
        .ok_or_else(|| "GITHUB_REPOSITORY must be owner/repo".to_string())?;
    let sha = std::env::var("GITHUB_SHA").unwrap_or_default();
    if sha.trim().is_empty() {
        return Err("GITHUB_SHA is missing".to_string());
    }
    std::env::var("GITHUB_TOKEN").map_err(|_| "GITHUB_TOKEN is missing".to_string())?;
    let prefix = if let Some(prefix) = std::env::var("E2E_BOARD_RUN_ID")
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
    {
        prefix
    } else {
        let run = std::env::var("GITHUB_RUN_ID").unwrap_or_default();
        if run.trim().is_empty() {
            return Err("GITHUB_RUN_ID is missing".to_string());
        }
        let attempt = std::env::var("GITHUB_RUN_ATTEMPT").unwrap_or_default();
        let attempt = attempt.trim();
        if attempt.is_empty() {
            run.trim().to_string()
        } else {
            format!("{}-{attempt}", run.trim())
        }
    };
    Ok(testing::harness::GitHubCheckpointBoard::new(
        owner.to_string(),
        repo.to_string(),
        sha.trim().to_string(),
        prefix,
    ))
}

#[cfg(test)]
mod tests {
    use super::try_from_env;

    #[test]
    fn test_usage() {
        unsafe {
            std::env::set_var("GITHUB_REPOSITORY", "owner/repo");
            std::env::set_var("GITHUB_SHA", "abc123");
            std::env::set_var("GITHUB_TOKEN", "token");
            std::env::set_var("E2E_BOARD_RUN_ID", "run-7");
        }
        let board = try_from_env().expect("board");
        assert!(board.commit_url().contains("owner/repo/commits/abc123"));
        unsafe {
            std::env::remove_var("GITHUB_REPOSITORY");
            std::env::remove_var("GITHUB_SHA");
            std::env::remove_var("GITHUB_TOKEN");
            std::env::remove_var("E2E_BOARD_RUN_ID");
        }
    }
}
