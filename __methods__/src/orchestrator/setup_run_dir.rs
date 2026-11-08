use std::path::PathBuf;

use crate::testing;

pub fn setup_run_dir(
    orchestrator: &testing::Orchestrator,
) -> Result<(PathBuf, String, String), String> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let room = format!("room-{stamp}");
    let token = format!("lobby-e2e/{stamp}");
    let forced = std::env::var("E2E_RUN_DIR").unwrap_or_default();
    let dir = if forced.trim().is_empty() {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".local-run")
            .join(format!("{}-{stamp}", orchestrator.game))
    } else {
        PathBuf::from(forced.trim())
    };
    std::fs::create_dir_all(&dir).map_err(|_| format!("create dir {} failed", dir.display()))?;
    Ok((dir, room, token))
}

#[cfg(test)]
mod tests {
    use super::setup_run_dir;
    use crate::testing;

    #[test]
    fn test_usage() {
        let orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::Fake,
        );
        let result = setup_run_dir(&orchestrator);
        assert!(result.is_ok() || result.is_err());
    }
}
