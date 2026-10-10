fn main() {
    println!("{}", token_hex());
}

#[must_use]
pub fn token_hex() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let digest = blake3::hash(format!("{nanos}-{}", std::process::id()).as_bytes());
    digest.to_hex().as_str().chars().take(16).collect()
}

#[cfg(test)]
mod tests {
    use super::token_hex;

    #[test]
    fn test_usage() {
        let token = token_hex();
        assert_eq!(token.len(), 16);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
