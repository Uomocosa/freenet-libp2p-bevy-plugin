#[must_use]
pub fn bench_date() -> String {
    std::process::Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map_or_else(|| String::from("unknown"), |text| text.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::bench_date;

    #[test]
    fn test_usage() {
        let date = bench_date();
        assert!(!date.is_empty());
        assert!(date.len() <= 32);
    }
}
