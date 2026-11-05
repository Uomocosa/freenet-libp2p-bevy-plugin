use std::fmt::Write as _;

use super::check_layer::CheckLayer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectationResult {
    pub layer: CheckLayer,
    pub name: String,
    pub expected: String,
    pub actual: String,
    pub pass: bool,
}

impl ExpectationResult {
    #[must_use]
    pub fn render_markdown_tables(results: &[Self]) -> String {
        let mut out = String::new();
        for layer in [
            CheckLayer::Scenario,
            CheckLayer::Freenet,
            CheckLayer::Libp2p,
        ] {
            let rows: Vec<&Self> = results.iter().filter(|row| row.layer == layer).collect();
            if rows.is_empty() {
                continue;
            }
            let title = match layer {
                CheckLayer::Scenario => "Scenario",
                CheckLayer::Freenet => "Freenet",
                CheckLayer::Libp2p => "libp2p",
            };
            let _ = writeln!(out, "## {title}");
            out.push_str("| name | expected | actual | PASS/FAIL |\n");
            out.push_str("|---|---|---|---|\n");
            for row in rows {
                let verdict = if row.pass { "PASS" } else { "FAIL" };
                let _ = writeln!(
                    out,
                    "| {} | {} | {} | {verdict} |",
                    row.name, row.expected, row.actual
                );
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::super::check_layer::CheckLayer;
    use super::ExpectationResult;

    #[test]
    fn test_usage() {
        let results = vec![ExpectationResult {
            layer: CheckLayer::Scenario,
            name: "canvas converged".to_string(),
            expected: "same hash".to_string(),
            actual: "same hash".to_string(),
            pass: true,
        }];
        let text = ExpectationResult::render_markdown_tables(&results);
        assert!(text.contains("## Scenario"));
        assert!(text.contains("PASS"));
    }

    mod render_markdown_tables {
        use super::super::super::check_layer::CheckLayer;
        use super::super::ExpectationResult;

        #[test]
        fn test_usage() {
            let results = vec![
                ExpectationResult {
                    layer: CheckLayer::Scenario,
                    name: "canvas converged".to_string(),
                    expected: "same hash".to_string(),
                    actual: "same hash".to_string(),
                    pass: true,
                },
                ExpectationResult {
                    layer: CheckLayer::Freenet,
                    name: "p1 lobby fetched".to_string(),
                    expected: "fetched".to_string(),
                    actual: "fetched".to_string(),
                    pass: false,
                },
            ];
            let text = ExpectationResult::render_markdown_tables(&results);
            assert!(text.contains("## Scenario"));
            assert!(text.contains("## Freenet"));
            assert!(!text.contains("## libp2p"));
            assert!(text.contains("| name | expected | actual | PASS/FAIL |"));
            assert!(text.contains("PASS"));
            assert!(text.contains("FAIL"));
        }
    }
}
