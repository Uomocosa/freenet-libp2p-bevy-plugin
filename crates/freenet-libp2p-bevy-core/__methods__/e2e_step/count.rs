pub fn count(verb: &str, arg: &str) -> Result<u32, String> {
    arg.trim()
        .parse::<u32>()
        .map_err(|_| format!("step '{verb}' needs a count, got '{arg}'"))
}

#[cfg(test)]
mod tests {
    use super::count;

    #[test]
    fn test_usage() {
        assert_eq!(count("wait-members", " 3 "), Ok(3));
        assert!(count("wait-members", "x").is_err());
    }
}
