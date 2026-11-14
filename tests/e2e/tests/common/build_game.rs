use std::path::{Path, PathBuf};
use std::process::Command;

/// Builds one example app (bin named after its package) and returns its path.
///
/// # Errors
/// Returns an error if the cargo build fails or the binary is missing.
pub fn build_game(example: &str) -> Result<PathBuf, String> {
    let profile = std::env::var("LOBBY_BUILD_PROFILE").unwrap_or_else(|_| "dev".to_string());
    build_package_bin(example, example, &profile)
}

/// Builds one binary of a workspace package and returns its path.
///
/// # Errors
/// Returns an error if cargo fails, cannot report its target dir, or the binary is missing.
pub fn build_package_bin(package: &str, bin: &str, profile: &str) -> Result<PathBuf, String> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest.join("../..");
    let target = target_root(&workspace)?;
    let mut build = Command::new("cargo");
    build.current_dir(&workspace);
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
        .args(["build", "-p", package, "--bin", bin, "--profile", profile])
        .output()
        .map_err(|e| format!("spawning cargo build: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo build failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let folder = if profile == "dev" { "debug" } else { profile };
    let path = target
        .join(folder)
        .join(format!("{bin}{}", std::env::consts::EXE_SUFFIX));
    if !path.exists() {
        return Err(format!(
            "expected build output {} not found",
            path.display()
        ));
    }
    Ok(path)
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let name = format!("starboard{}", std::env::consts::EXE_SUFFIX);
        #[cfg(windows)]
        assert_eq!(name, "starboard.exe");
        #[cfg(not(windows))]
        assert_eq!(name, "starboard");
    }
}
