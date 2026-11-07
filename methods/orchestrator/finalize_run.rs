use crate::testing;

#[must_use]
pub fn finalize_run(
    game: &str,
    room: &str,
    transport: &str,
    dir: std::path::PathBuf,
    running: &mut [testing::RunningPeer],
    checks: Vec<testing::ReportCheck>,
) -> testing::OrchestratorReport {
    for slot in running.iter_mut() {
        if let Some(child) = slot.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
    let mut lines = vec![format!(
        "{game} room={room} transport={transport} peers={}",
        running.len()
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
    let caption = lines.join("\n");
    testing::OrchestratorReport {
        checks,
        logs_dir: dir,
        caption,
    }
}

#[cfg(test)]
mod tests {
    use super::finalize_run;
    use std::path::PathBuf;

    use crate::testing;

    #[test]
    fn test_usage() {
        let mut running: Vec<testing::RunningPeer> = Vec::new();
        let report = finalize_run(
            "starboard",
            "room-1",
            "both",
            PathBuf::from("/tmp/run"),
            &mut running,
            Vec::new(),
        );
        assert!(report.all_ok());
    }
}
