use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tick {
    pub catalogue: Vec<String>,
    pub room: Option<String>,
    pub members: usize,
    pub connected: usize,
}

#[must_use]
pub fn log_contains(path: &Path, needle: &str) -> bool {
    strip_ansi(&std::fs::read_to_string(path).unwrap_or_default()).contains(needle)
}

#[must_use]
pub fn log_matches(path: &Path, needle: &str) -> usize {
    strip_ansi(&std::fs::read_to_string(path).unwrap_or_default())
        .lines()
        .filter(|line| line.contains(needle))
        .count()
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

#[must_use]
pub fn last_tick(path: &Path) -> Option<Tick> {
    let content = strip_ansi(&std::fs::read_to_string(path).ok()?);
    content.lines().filter_map(parse_tick_line).next_back()
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

// needed helper: parses one `lobby tick ...` marker line
fn parse_tick_line(line: &str) -> Option<Tick> {
    let start = line.find("lobby tick ")?;
    let rest = line.get(start.checked_add("lobby tick ".len())?..)?;
    let mut tick = Tick {
        catalogue: Vec::new(),
        room: None,
        members: 0,
        connected: 0,
    };
    for token in rest.split_whitespace() {
        if let Some(value) = token.strip_prefix("catalogue=") {
            if value != "none" {
                tick.catalogue = value
                    .split(',')
                    .filter(|name| !name.is_empty())
                    .map(str::to_string)
                    .collect();
            }
        } else if let Some(value) = token.strip_prefix("room=") {
            tick.room = (value != "none").then(|| value.to_string());
        } else if let Some(value) = token.strip_prefix("members=") {
            tick.members = value.parse().unwrap_or(0);
        } else if let Some(value) = token.strip_prefix("connected=") {
            tick.connected = value.parse().unwrap_or(0);
        }
    }
    Some(tick)
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
