use std::path::{Path, PathBuf};
use std::process::Command;

/// # Errors
/// Returns an error if the cargo build fails or the binary is missing.
pub fn build_game() -> Result<PathBuf, String> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let target = target_root(manifest);
    let profile = std::env::var("LOBBY_BUILD_PROFILE").unwrap_or_else(|_| "dev".to_string());
    let output = Command::new("cargo")
        .current_dir(manifest)
        .env("CARGO_TARGET_DIR", &target)
        .args([
            "build",
            "--example",
            "lobby_room",
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
    let bin = target.join(folder).join("examples").join("lobby_room");
    if !bin.exists() {
        return Err(format!("expected build output {} not found", bin.display()));
    }
    Ok(bin)
}

// needed helper: honours devenv's shared target dir, else falls back to ./target
fn target_root(manifest: &Path) -> PathBuf {
    std::env::var("CARGO_TARGET_DIR").map_or_else(|_| manifest.join("target"), PathBuf::from)
}
