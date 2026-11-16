use crate::testing;

pub fn reused_display_name() -> Option<String> {
    std::env::var(testing::harness::E2E_DISPLAY_ENV)
        .ok()
        .filter(|name| !name.is_empty())
}

#[cfg(test)]
mod tests {
    use super::reused_display_name;

    #[test]
    fn test_usage() {
        assert!(reused_display_name().is_none_or(|name| name.starts_with(':')));
    }
}
