use crate::testing;

pub async fn post_status(
    board: &testing::harness::GitHubCheckpointBoard,
    context: &str,
    description: &str,
) -> Result<(), String> {
    let url = format!("{}/statuses", board.commit_url());
    let response = board
        .client
        .post(&url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .bearer_auth(testing::harness::GitHubCheckpointBoard::token()?)
        .json(&serde_json::json!({
            "state": "success",
            "context": context,
            "description": description,
        }))
        .send()
        .await
        .map_err(|error| format!("post status {context}: {error}"))?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("post status {context}: {}", response.status()))
    }
}

// no test_usage necessary
