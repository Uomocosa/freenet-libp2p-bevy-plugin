use std::path::{Path, PathBuf};
use std::process::Command;

#[must_use]
pub fn speed_clip(input: &Path, output: &Path) -> Option<PathBuf> {
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-filter:v",
            "setpts=0.5*PTS",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-crf",
            "30",
            "-pix_fmt",
            "yuv420p",
            &output.to_string_lossy(),
        ])
        .status()
        .ok()?;
    if status.success() && output.exists() {
        Some(output.to_path_buf())
    } else {
        None
    }
}
