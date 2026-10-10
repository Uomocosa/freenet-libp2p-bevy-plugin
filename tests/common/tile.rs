use std::process::Command;
use std::time::Duration;

use super::terminal_guard::TerminalGuard;

/// # Errors
/// Returns an error if any window cannot be found and positioned.
pub fn tile(windows: &[TerminalGuard]) -> Result<(), String> {
    let targets = layout(windows.len());
    for (index, (w, h, x, y)) in targets.iter().enumerate() {
        let Some(window) = windows.get(index) else {
            continue;
        };
        place(&window.window_title, *w, *h, *x, *y)?;
    }
    std::thread::sleep(Duration::from_millis(400));
    Ok(())
}

// needed helper: fixed grid layouts for 3 and 5 windows on a 1920x1080 screen
fn layout(count: usize) -> Vec<(u32, u32, i32, i32)> {
    match count {
        3 => vec![(960, 540, 0, 0), (960, 540, 0, 540), (960, 1080, 960, 0)],
        5 => vec![
            (640, 540, 0, 0),
            (640, 540, 640, 0),
            (640, 540, 1280, 0),
            (960, 540, 0, 540),
            (960, 540, 960, 540),
        ],
        other => (0..other)
            .map(|i| {
                let index = i32::try_from(i).unwrap_or(0);
                let col = index.checked_rem(4).unwrap_or(0);
                let row = index.checked_div(4).unwrap_or(0);
                (480, 360, col.saturating_mul(480), row.saturating_mul(360))
            })
            .collect(),
    }
}

// needed helper: positions one window by title, failing if it never appears
fn place(title: &str, w: u32, h: u32, x: i32, y: i32) -> Result<(), String> {
    for _ in 0..30 {
        if let Some(id) = find_window_id(title) {
            let _ = Command::new("xdotool")
                .args(["windowsize", &id, &w.to_string(), &h.to_string()])
                .output();
            let _ = Command::new("xdotool")
                .args(["windowmove", &id, &x.to_string(), &y.to_string()])
                .output();
            let _ = Command::new("wmctrl")
                .args(["-r", title, "-e", &format!("0,{x},{y},{w},{h}")])
                .output();
            let _ = Command::new("xdotool")
                .args(["windowactivate", &id])
                .output();
            let _ = Command::new("wmctrl").args(["-a", title]).output();
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    Err(format!("x11 window not found for title {title:?}"))
}

// needed helper: resolves an x11 window id by title via xdotool then wmctrl
fn find_window_id(title: &str) -> Option<String> {
    if let Ok(out) = Command::new("xdotool")
        .args(["search", "--onlyvisible", "--name", title])
        .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        if let Some(first) = text.lines().next().map(str::trim).filter(|s| !s.is_empty()) {
            return Some(first.to_string());
        }
    }
    let out = Command::new("wmctrl").args(["-l"]).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if line.contains(title) {
            return line.split_whitespace().next().map(str::to_string);
        }
    }
    None
}
