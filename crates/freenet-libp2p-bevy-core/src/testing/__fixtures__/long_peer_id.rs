#[must_use]
pub fn long_peer_id() -> String {
    format!("12D3Koo{}", "a".repeat(45))
}

#[cfg(test)]
mod tests {
    use super::long_peer_id;

    #[test]
    fn test_usage() {
        assert_eq!(long_peer_id().len(), 52);
    }
}
