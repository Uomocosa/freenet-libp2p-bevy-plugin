use std::time::Duration;

pub fn parse_seconds(raw: &str) -> Result<Duration, String> {
    raw.trim()
        .strip_suffix('s')
        .and_then(|secs| secs.parse::<u64>().ok())
        .map(Duration::from_secs)
        .ok_or_else(|| format!("expected seconds like '30s', got '{raw}'"))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::parse_seconds;

    #[test]
    fn test_usage() {
        assert_eq!(parse_seconds("45s"), Ok(Duration::from_secs(45)));
        assert!(parse_seconds("45").is_err());
    }
}
