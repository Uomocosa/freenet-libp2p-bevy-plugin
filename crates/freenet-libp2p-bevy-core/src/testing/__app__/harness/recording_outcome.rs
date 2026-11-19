#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RecordingOutcome {
    #[default]
    Failed,
    Passed,
}

impl std::fmt::Display for RecordingOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Failed => write!(formatter, "FAILED"),
            Self::Passed => write!(formatter, "PASSED"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RecordingOutcome;

    #[test]
    fn test_usage() {
        assert_eq!(RecordingOutcome::default(), RecordingOutcome::Failed);
        assert_eq!(RecordingOutcome::Failed.to_string(), "FAILED");
        assert_eq!(RecordingOutcome::Passed.to_string(), "PASSED");
    }
}
