pub fn check_results_dir() -> Result<std::path::PathBuf, String> {
    let raw = std::env::var("E2E_RESULTS_DIR").unwrap_or_default();
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("E2E_ROLE=check needs E2E_RESULTS_DIR".to_string());
    }
    Ok(std::path::PathBuf::from(trimmed))
}

#[cfg(test)]
mod tests {
    use super::check_results_dir;

    #[test]
    fn test_usage() {
        let expected = std::env::var("E2E_RESULTS_DIR")
            .ok()
            .map(|raw| raw.trim().to_string())
            .filter(|trimmed| !trimmed.is_empty());
        match expected {
            None => assert!(check_results_dir().is_err()),
            Some(expected) => assert_eq!(
                check_results_dir().expect("dir"),
                std::path::PathBuf::from(expected)
            ),
        }
    }
}
