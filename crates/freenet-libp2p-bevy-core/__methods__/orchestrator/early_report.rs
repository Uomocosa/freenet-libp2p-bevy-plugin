use std::path::PathBuf;

use crate::testing;

pub fn early_report(game: &str, dir: PathBuf, detail: String) -> testing::OrchestratorReport {
    testing::OrchestratorReport {
        checks: vec![testing::ReportCheck {
            name: "setup".to_string(),
            ok: false,
            detail,
        }],
        logs_dir: dir,
        caption: format!("{game} setup failed"),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::early_report;

    #[test]
    fn test_usage() {
        let report = early_report("starboard", PathBuf::from("logs"), "boom".to_string());
        assert_eq!(report.checks.len(), 1);
        assert!(!report.checks[0].ok);
        assert_eq!(report.checks[0].detail, "boom");
        assert_eq!(report.caption, "starboard setup failed");
        assert_eq!(report.logs_dir, PathBuf::from("logs"));
    }
}
