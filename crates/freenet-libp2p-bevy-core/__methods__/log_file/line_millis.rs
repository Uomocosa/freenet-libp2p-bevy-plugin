pub fn line_millis(line: &str) -> Option<u64> {
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

#[cfg(test)]
mod tests {
    use super::line_millis;

    #[test]
    fn test_usage() {
        assert_eq!(
            line_millis("2026-10-01T17:04:19.867Z hello"),
            Some(17 * 3_600_000 + 4 * 60_000 + 19_867)
        );
        assert_eq!(line_millis("no timestamp here"), None);
    }
}
