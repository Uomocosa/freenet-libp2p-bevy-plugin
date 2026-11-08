use std::path::Path;

use crate::testing;

#[must_use]
pub fn check_peer_dir(
    orchestrator: &testing::Orchestrator,
    dir: &Path,
    name: &str,
) -> testing::ReportCheck {
    let fail = |detail: String| testing::ReportCheck {
        name: name.to_string(),
        ok: false,
        detail,
    };
    let peer_dir = dir.join(name);
    if !peer_dir.join("result.json").exists() {
        return fail("missing result.json".to_string());
    }
    let mut logs: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(file) = path.file_name().and_then(|file| file.to_str()) else {
                continue;
            };
            if file.starts_with(&format!("instance-{name}")) {
                logs.push(path);
            }
        }
    }
    if logs.is_empty() {
        return fail("no instance logs".to_string());
    }
    let mut failed = false;
    let mut done = false;
    let mut freenet_started = false;
    for log in &logs {
        let text = std::fs::read_to_string(log).unwrap_or_default();
        failed = failed || text.contains(testing::SCRIPT_FAILED_MARKER);
        done = done || text.contains(testing::SCRIPT_DONE_MARKER);
        freenet_started = freenet_started || text.contains("lobby freenet node started");
    }
    if failed {
        return fail("a log has the script failed marker".to_string());
    }
    if orchestrator.should_forbid_freenet_node && freenet_started {
        return fail("the embedded freenet node started".to_string());
    }
    let stays = orchestrator
        .peers
        .iter()
        .rev()
        .find(|peer| peer.name == *name)
        .is_some_and(|peer| peer.steps.last().is_some_and(|step| step.text == "stay"));
    if !stays && !done {
        return fail("no log has the script done marker".to_string());
    }
    testing::ReportCheck {
        name: name.to_string(),
        ok: true,
        detail: format!("logs={} done={done}", logs.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::check_peer_dir;
    use crate::testing;

    fn orchestrator() -> testing::Orchestrator {
        let mut orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::DirectDial,
        );
        orchestrator.peer("alice");
        orchestrator.expect_no_freenet_node();
        orchestrator
    }

    fn write_run(dir: &std::path::Path, log: &str) {
        std::fs::create_dir_all(dir.join("alice")).expect("peer dir");
        std::fs::write(dir.join("alice").join("result.json"), "{}").expect("result");
        std::fs::write(dir.join("instance-alice.log"), log).expect("log");
    }

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_run(dir.path(), "script done\n");
        let check = check_peer_dir(&orchestrator(), dir.path(), "alice");
        assert!(check.ok, "{}", check.detail);
        assert!(!check_peer_dir(&orchestrator(), dir.path(), "mallory").ok);
    }

    #[test]
    fn failed_and_freenet_markers_fail() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_run(dir.path(), "script failed\n");
        assert!(!check_peer_dir(&orchestrator(), dir.path(), "alice").ok);
        write_run(dir.path(), "script done\nlobby freenet node started\n");
        assert!(!check_peer_dir(&orchestrator(), dir.path(), "alice").ok);
    }

    #[test]
    fn unfinished_scripts_fail_unless_they_stay() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_run(dir.path(), "some output\n");
        assert!(!check_peer_dir(&orchestrator(), dir.path(), "alice").ok);
        let mut staying = orchestrator();
        staying.peers[0].steps.push(testing::StoredStep {
            text: "stay".to_string(),
            is_wait: false,
            within: None,
        });
        assert!(check_peer_dir(&staying, dir.path(), "alice").ok);
    }
}
