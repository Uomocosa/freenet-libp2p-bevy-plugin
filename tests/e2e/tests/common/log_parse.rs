use std::io::Read;
use std::path::Path;

#[must_use]
pub fn log_contains(path: &Path, needle: &str) -> bool {
    strip_ansi(&std::fs::read_to_string(path).unwrap_or_default()).contains(needle)
}

/// Reads at most `max_bytes` from the head of a log (ANSI stripped), so huge
/// run logs can be inspected for early bootstrap markers without loading GBs.
#[must_use]
pub fn read_head(path: &Path, max_bytes: u64) -> String {
    let Ok(file) = std::fs::File::open(path) else {
        return String::new();
    };
    let mut buf = Vec::new();
    let _ = file.take(max_bytes).read_to_end(&mut buf);
    strip_ansi(&String::from_utf8_lossy(&buf))
}

/// Milliseconds-of-day timestamps of every line containing `needle`.
#[must_use]
pub fn marker_times(path: &Path, needle: &str) -> Vec<(u64, String)> {
    strip_ansi(&std::fs::read_to_string(path).unwrap_or_default())
        .lines()
        .filter(|line| line.contains(needle))
        .filter_map(|line| line_millis(line).map(|ms| (ms, line.to_string())))
        .collect()
}

// needed helper: `2026-10-01T17:04:19.867931Z ...` -> milliseconds since midnight
fn line_millis(line: &str) -> Option<u64> {
    let time = line.split_whitespace().next()?.split('T').nth(1)?;
    let time = time.trim_end_matches('Z');
    let (hms, frac) = time.split_once('.')?;
    let mut parts = hms.split(':').map(|part| part.parse::<u64>().ok());
    let hours = parts.next()??;
    let minutes = parts.next()??;
    let seconds = parts.next()??;
    let millis = frac.get(..3)?.parse::<u64>().ok()?;
    hours
        .checked_mul(60)?
        .checked_add(minutes)?
        .checked_mul(60)?
        .checked_add(seconds)?
        .checked_mul(1000)?
        .checked_add(millis)
}

// needed helper: removes ANSI escapes so markers survive colored tracing output
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for cc in chars.by_ref() {
                if cc == 'm' {
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}
