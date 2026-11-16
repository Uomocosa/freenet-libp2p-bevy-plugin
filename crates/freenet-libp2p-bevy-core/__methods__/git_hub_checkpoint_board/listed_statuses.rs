use std::time::Duration;

use crate::testing;

const STATUS_CACHE_TTL: Duration = Duration::from_secs(10);

pub async fn listed_statuses(
    board: &testing::harness::GitHubCheckpointBoard,
) -> Result<Vec<testing::harness::BoardStatus>, String> {
    {
        let cached = board.cached.lock().await;
        if let Some(cached) = cached.as_ref()
            && cached.is_fresh(STATUS_CACHE_TTL)
        {
            return Ok(cached.statuses.clone());
        }
    }
    let url = format!("{}/status?per_page=100", board.commit_url());
    let response = board
        .client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .bearer_auth(testing::harness::GitHubCheckpointBoard::token()?)
        .send()
        .await
        .map_err(|error| format!("list statuses: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("list statuses: {}", response.status()));
    }
    let combined: serde_json::Value = response
        .json()
        .await
        .map_err(|error| format!("decode statuses: {error}"))?;
    let statuses: Vec<testing::harness::BoardStatus> = combined
        .get("statuses")
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .map_err(|error| format!("decode statuses: {error}"))?
        .unwrap_or_default();
    *board.cached.lock().await = Some(testing::harness::CachedStatuses {
        at: std::time::Instant::now(),
        statuses: statuses.clone(),
    });
    Ok(statuses)
}

// no test_usage necessary
