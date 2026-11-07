use std::path::PathBuf;

use crate::testing;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestratorReport {
    pub checks: Vec<testing::ReportCheck>,
    pub logs_dir: PathBuf,
    pub caption: String,
}

#[rustfmt::skip]
impl OrchestratorReport {
    #[must_use]
    pub fn all_ok(&self) -> bool { self.checks.iter().all(|check| check.ok) }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::OrchestratorReport;
    use crate::testing;

    #[test]
    fn test_usage() {
        let report = OrchestratorReport {
            checks: vec![testing::ReportCheck {
                name: "p1".to_string(),
                ok: true,
                detail: "exit=0".to_string(),
            }],
            logs_dir: PathBuf::from("/tmp/run"),
            caption: "starboard_e2e".to_string(),
        };
        assert!(report.all_ok());
        assert_eq!(report.checks.len(), 1);
    }
}
