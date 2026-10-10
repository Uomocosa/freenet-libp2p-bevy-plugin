use std::path::Path;
use std::process::{Child, Command, Stdio};

const DEFAULT_RECORD_SECS: u64 = 600;

#[must_use]
pub fn start_record(clip_secs: u64, path: &Path) -> Option<Child> {
    let secs = std::env::var("LOBBY_RECORD_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or_else(|| clip_secs.max(DEFAULT_RECORD_SECS));
    let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0.0".to_string());
    let size = display_size().unwrap_or_else(|| "1920x1080".to_string());
    let path_str = path.to_string_lossy().to_string();
    Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "x11grab",
            "-framerate",
            "15",
            "-video_size",
            &size,
            "-i",
            &display,
            "-t",
            &secs.to_string(),
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-crf",
            "30",
            "-pix_fmt",
            "yuv420p",
            &path_str,
        ])
        .stdin(Stdio::piped())
        .spawn()
        .ok()
}

// needed helper: reads the X screen geometry so x11grab matches the display
fn display_size() -> Option<String> {
    let out = Command::new("xdpyinfo").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if line.contains("dimensions:") {
            let part = line.split("dimensions:").nth(1)?;
            return part.split_whitespace().next().map(str::to_string);
        }
    }
    None
}
