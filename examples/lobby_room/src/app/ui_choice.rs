#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiChoice {
    Default,
    Custom,
}

#[cfg(test)]
mod tests {
    use super::UiChoice;

    #[test]
    fn test_usage() {
        assert_ne!(UiChoice::Default, UiChoice::Custom);
    }
}
