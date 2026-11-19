use crate::testing;

#[must_use]
pub fn board_status(context: &str, description: &str) -> testing::harness::BoardStatus {
    testing::harness::BoardStatus {
        context: context.to_string(),
        description: Some(description.to_string()),
        state: "success".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::board_status;

    #[test]
    fn test_usage() {
        assert!(board_status("run-1/checkpoint/done", "done").is_success("run-1/checkpoint/done"));
    }
}
