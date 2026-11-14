use std::path::PathBuf;
use std::process::Command;

/// Builds one binary of a workspace package and returns its path.
///
/// # Errors
/// Returns an error if cargo fails, cannot report its target dir, or the binary is missing.
pub fn build_package_bin(package: &str, bin: &str) -> Result<PathBuf, String> {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let target = target_directory(&workspace)?;
    let mut build = Command::new("cargo");
    build
        .current_dir(&workspace)
        .env("CARGO_TARGET_DIR", &target);
    for (key, _) in std::env::vars() {
        if key == "CARGO_TARGET_DIR" {
            continue;
        }
        if key.starts_with("CARGO_") || key.starts_with("NEXTEST_") || key == "RUST_RECURSION_COUNT"
        {
            build.env_remove(key);
        }
    }
    let output = build
        .args(["build", "-p", package, "--bin", bin])
        .output()
        .map_err(|e| format!("spawning cargo build: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo build -p {package} --bin {bin} failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let path = target
        .join("debug")
        .join(format!("{bin}{}", std::env::consts::EXE_SUFFIX));
    if !path.exists() {
        return Err(format!(
            "expected build output {} not found",
            path.display()
        ));
    }
    Ok(path)
}

fn target_directory(workspace: &std::path::Path) -> Result<PathBuf, String> {
    if let Ok(dir) = std::env::var("CARGO_TARGET_DIR") {
        return Ok(PathBuf::from(dir));
    }
    let output = Command::new("cargo")
        .current_dir(workspace)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .map_err(|e| format!("spawning cargo metadata: {e}"))?;
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("cargo metadata output: {e}"))?;
    metadata
        .get("target_directory")
        .and_then(serde_json::Value::as_str)
        .map(PathBuf::from)
        .ok_or_else(|| "cargo metadata: target_directory missing".to_string())
}
