use std::path::Path;

use crate::testing;

pub async fn run_check_role(
    orchestrator: &testing::Orchestrator,
    dir: &Path,
) -> testing::OrchestratorReport {
    let mut ordered: Vec<String> = Vec::new();
    for peer in &orchestrator.peers {
        if !ordered.iter().any(|known| known == &peer.name) {
            ordered.push(peer.name.clone());
        }
    }
    let mut checks: Vec<testing::ReportCheck> = Vec::new();
    for name in &ordered {
        checks.push(orchestrator.check_peer_dir(dir, name));
    }
    let mut lines = vec![format!(
        "{} check peers={}",
        orchestrator.game,
        ordered.len()
    )];
    for check in &checks {
        lines.push(format!(
            "{} {} {}",
            if check.ok { "✅" } else { "❌" },
            check.name,
            check.detail
        ));
    }
    lines.push(format!("logs: {}", dir.display()));
    testing::OrchestratorReport {
        checks,
        caption: lines.join("\n"),
        logs_dir: dir.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::run_check_role;
    use crate::testing;

    fn orchestrator() -> testing::Orchestrator {
        let mut orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::DirectDial,
        );
        orchestrator.peer("alice");
        orchestrator.expect_no_log_line(testing::FREENET_NODE_STARTED);
        orchestrator
    }

    fn write_run(dir: &std::path::Path, log: &str) {
        std::fs::create_dir_all(dir.join("alice")).expect("peer dir");
        std::fs::write(dir.join("alice").join("result.json"), "{}").expect("result");
        std::fs::write(dir.join("instance-alice.log"), log).expect("log");
    }

    #[tokio::test]
    async fn test_usage() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_run(dir.path(), "script done\n");
        let report = run_check_role(&orchestrator(), dir.path()).await;
        assert!(report.all_ok(), "{}", report.caption);
        assert_eq!(report.checks.len(), 1);
    }

    #[tokio::test]
    async fn missing_result_is_a_failure() {
        let dir = tempfile::tempdir().expect("tempdir");
        let report = run_check_role(&orchestrator(), dir.path()).await;
        assert!(!report.all_ok());
    }

    #[tokio::test]
    async fn failed_and_freenet_markers_fail() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_run(dir.path(), "script failed\n");
        assert!(!run_check_role(&orchestrator(), dir.path()).await.all_ok());
        write_run(
            dir.path(),
            &format!("script done\n{}\n", testing::FREENET_NODE_STARTED),
        );
        assert!(!run_check_role(&orchestrator(), dir.path()).await.all_ok());
    }

    #[tokio::test]
    async fn unfinished_scripts_fail_unless_they_stay() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_run(dir.path(), "some output\n");
        assert!(!run_check_role(&orchestrator(), dir.path()).await.all_ok());
        let mut staying = orchestrator();
        staying.peers[0].steps.push(testing::StoredStep {
            text: "stay".to_string(),
            is_wait: false,
            within: None,
        });
        assert!(run_check_role(&staying, dir.path()).await.all_ok());
    }
}
