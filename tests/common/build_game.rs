use std::path::{Path, PathBuf};
use std::process::Command;

/// # Errors
/// Returns an error if the cargo build fails or the binary is missing.
pub fn build_game(example: &str) -> Result<PathBuf, String> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let target = target_root(manifest)?;
    let profile = std::env::var("LOBBY_BUILD_PROFILE").unwrap_or_else(|_| "dev".to_string());
    let output = Command::new("cargo")
        .current_dir(manifest)
        .env("CARGO_TARGET_DIR", &target)
        .args([
            "build",
            "--example",
            example,
            "--features",
            "e2e",
            "--profile",
            &profile,
        ])
        .output()
        .map_err(|e| format!("spawning cargo build: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo build failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let folder = if profile == "dev" {
        "debug"
    } else {
        profile.as_str()
    };
    let bin = target.join(folder).join("examples").join(example);
    if !bin.exists() {
        return Err(format!("expected build output {} not found", bin.display()));
    }
    Ok(bin)
}

// needed helper: cargo's real target dir (honours .cargo/config.toml target-dir), else ./target
fn target_root(manifest: &Path) -> Result<PathBuf, String> {
    const KEY: &str = "\"target_directory\":\"";
    if let Ok(dir) = std::env::var("CARGO_TARGET_DIR") {
        return Ok(PathBuf::from(dir));
    }
    let output = Command::new("cargo")
        .current_dir(manifest)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .map_err(|e| format!("spawning cargo metadata: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let json = String::from_utf8_lossy(&output.stdout);
    let start = json
        .find(KEY)
        .ok_or_else(|| "cargo metadata: target_directory missing".to_string())?
        .saturating_add(KEY.len());
    let end = json
        .get(start..)
        .and_then(|rest| rest.find('"'))
        .map_or(start, |offset| start.saturating_add(offset));
    json.get(start..end)
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "cargo metadata: malformed target_directory".to_string())
}
