use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BoardStatus {
    pub context: String,
    pub description: Option<String>,
    pub state: String,
}

#[rustfmt::skip]
impl BoardStatus {
    #[must_use]
    pub fn is_success(&self, context: &str) -> bool { self.context == context && self.state == "success" }
}

#[cfg(test)]
mod tests {
    use super::BoardStatus;

    #[test]
    fn test_usage() {
        let status: BoardStatus = serde_json::from_value(serde_json::json!({
            "context": "run-1/checkpoint/done",
            "description": "checkpoint done completed",
            "state": "success",
        }))
        .expect("decode");
        assert!(status.is_success("run-1/checkpoint/done"));
        assert!(!status.is_success("run-1/checkpoint/other"));
        let pending: BoardStatus = serde_json::from_value(serde_json::json!({
            "context": "run-1/checkpoint/done",
            "state": "pending",
        }))
        .expect("decode");
        assert_eq!(pending.description, None);
        assert!(!pending.is_success("run-1/checkpoint/done"));
    }
}
