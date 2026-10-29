use atomic_delegate_macros::atomic_delegates;

use crate::e2e_report;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectationResult {
    pub layer: e2e_report::CheckLayer,
    pub name: String,
    pub expected: String,
    pub actual: String,
    pub pass: bool,
}

#[atomic_delegates]
impl ExpectationResult {
    pub fn render_markdown_tables(results: &[Self]) -> String {}
}

#[cfg(test)]
mod tests {
    use super::ExpectationResult;
    use crate::e2e_report;

    #[test]
    fn test_usage() {
        let results = vec![ExpectationResult {
            layer: e2e_report::CheckLayer::Scenario,
            name: "canvas converged".to_string(),
            expected: "same hash".to_string(),
            actual: "same hash".to_string(),
            pass: true,
        }];
        let text = ExpectationResult::render_markdown_tables(&results);
        assert!(text.contains("## Scenario"));
        assert!(text.contains("PASS"));
    }
}
