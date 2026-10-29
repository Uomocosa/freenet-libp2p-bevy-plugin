use std::fmt::Write as _;

use crate::e2e_report;

#[must_use]
pub fn render_markdown_tables(results: &[e2e_report::ExpectationResult]) -> String {
    let mut out = String::new();
    for layer in [
        e2e_report::CheckLayer::Scenario,
        e2e_report::CheckLayer::Freenet,
        e2e_report::CheckLayer::Libp2p,
    ] {
        let rows: Vec<&e2e_report::ExpectationResult> =
            results.iter().filter(|row| row.layer == layer).collect();
        if rows.is_empty() {
            continue;
        }
        let title = match layer {
            e2e_report::CheckLayer::Scenario => "Scenario",
            e2e_report::CheckLayer::Freenet => "Freenet",
            e2e_report::CheckLayer::Libp2p => "libp2p",
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

#[cfg(test)]
mod tests {
    use super::render_markdown_tables;
    use crate::e2e_report;

    #[test]
    fn test_usage() {
        let results = vec![
            e2e_report::ExpectationResult {
                layer: e2e_report::CheckLayer::Scenario,
                name: "canvas converged".to_string(),
                expected: "same hash".to_string(),
                actual: "same hash".to_string(),
                pass: true,
            },
            e2e_report::ExpectationResult {
                layer: e2e_report::CheckLayer::Freenet,
                name: "p1 lobby attached".to_string(),
                expected: "attached".to_string(),
                actual: "attached".to_string(),
                pass: false,
            },
        ];
        let text = render_markdown_tables(&results);
        assert!(text.contains("## Scenario"));
        assert!(text.contains("## Freenet"));
        assert!(!text.contains("## libp2p"));
        assert!(text.contains("| name | expected | actual | PASS/FAIL |"));
        assert!(text.contains("PASS"));
        assert!(text.contains("FAIL"));
    }
}
