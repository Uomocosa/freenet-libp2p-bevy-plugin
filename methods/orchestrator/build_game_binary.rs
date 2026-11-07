use std::path::PathBuf;
use std::process::Command;

use crate::testing;

pub fn build_game_binary(orchestrator: &testing::Orchestrator) -> Result<PathBuf, String> {
    if let Ok(path) = std::env::var("E2E_GAME_BIN") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target = {
        const KEY: &str = "\"target_directory\":\"";
        let from_env = std::env::var("CARGO_TARGET_DIR")
            .ok()
            .filter(|dir| !dir.trim().is_empty())
            .map(PathBuf::from);
        if let Some(dir) = from_env {
            dir
        } else {
            let output = Command::new("cargo")
                .current_dir(&manifest)
                .args(["metadata", "--format-version", "1", "--no-deps"])
                .output()
                .map_err(|error| format!("spawning cargo metadata: {error}"))?;
            if !output.status.success() {
                return Err(format!(
                    "cargo metadata failed:\n{}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            let json = String::from_utf8_lossy(&output.stdout).to_string();
            let Some(start_index) = json.find(KEY) else {
                return Err("cargo metadata: target_directory missing".to_string());
            };
            let start = start_index.saturating_add(KEY.len());
            let rest = json.get(start..).unwrap_or("");
            let Some(end_offset) = rest.find('"') else {
                return Err("cargo metadata: malformed target_directory".to_string());
            };
            let end = start.saturating_add(end_offset);
            let Some(dir) = json.get(start..end) else {
                return Err("cargo metadata: malformed target_directory".to_string());
            };
            if dir.is_empty() {
                return Err("cargo metadata: malformed target_directory".to_string());
            }
            PathBuf::from(dir)
        }
    };
    let profile = std::env::var("LOBBY_BUILD_PROFILE").unwrap_or_else(|_| "dev".to_string());
    let mut build = Command::new("cargo");
    build.current_dir(&manifest);
    build.env("CARGO_TARGET_DIR", &target);
    for (key, _) in std::env::vars() {
        if key == "CARGO_TARGET_DIR" {
            continue;
        }
        if key.starts_with("CARGO_")
            || key.starts_with("NEXTEST_")
            || key == "RUST_RECURSION_COUNT"
            || key == "LD_LIBRARY_PATH"
        {
            build.env_remove(key);
        }
    }
    let output = build
        .args([
            "build",
            "--example",
            orchestrator.game.as_str(),
            "--features",
            "e2e",
            "--profile",
            profile.as_str(),
        ])
        .output()
        .map_err(|error| format!("spawning cargo build: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo build failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let folder = if profile == "dev" {
        "debug".to_string()
    } else {
        profile.clone()
    };
    let bin = target
        .join(folder)
        .join("examples")
        .join(&orchestrator.game);
    if bin.exists() {
        Ok(bin)
    } else {
        Err(format!("expected build output {} not found", bin.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::build_game_binary;
    use crate::testing;

    #[test]
    fn test_usage() {
        let orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::Fake,
        );
        let _ = build_game_binary(&orchestrator).map(|_| ());
    }
}
