use std::path::{Path, PathBuf};
use std::process::Command;

use crate::testing;

pub fn fit_under_budget(clip: &Path) -> Result<PathBuf, String> {
    let bytes = std::fs::metadata(clip)
        .map_err(|error| format!("clip {}: {error}", clip.display()))?
        .len();
    if bytes <= testing::harness::TELEGRAM_VIDEO_BYTES {
        return Ok(clip.to_path_buf());
    }
    let smaller = clip.with_extension("small.mp4");
    let mut arguments = ["-y", "-hide_banner", "-loglevel", "error", "-i"]
        .map(str::to_string)
        .to_vec();
    arguments.push(clip.display().to_string());
    arguments.extend(
        [
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            testing::harness::E2E_RECORDING_RECODE_CRF,
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
        ]
        .map(str::to_string),
    );
    arguments.push(smaller.display().to_string());
    let binary = testing::harness::RunRecording::ffmpeg_binary();
    let status = Command::new(&binary)
        .args(&arguments)
        .status()
        .map_err(|error| format!("re-encoding {} with {binary}: {error}", clip.display()))?;
    if !status.success() {
        return Err(format!("re-encoding {} failed: {status}", clip.display()));
    }
    let smaller_bytes = std::fs::metadata(&smaller)
        .map_err(|error| format!("clip {}: {error}", smaller.display()))?
        .len();
    if smaller_bytes <= testing::harness::TELEGRAM_VIDEO_BYTES {
        return Ok(smaller);
    }
    Err(format!(
        "{} is still {smaller_bytes} bytes after re-encoding; skipped",
        smaller.display()
    ))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::fit_under_budget;

    #[test]
    fn test_usage() {
        let missing = fit_under_budget(Path::new("/nonexistent-e2e-clip.mp4"));
        assert!(missing.is_err());
    }
}
