use std::time::{SystemTime, UNIX_EPOCH};

pub fn now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos())
}

#[cfg(test)]
mod tests {
    use super::now;

    #[test]
    fn test_usage() {
        assert!(now() > 0);
    }
}
