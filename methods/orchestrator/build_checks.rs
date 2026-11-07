use crate::testing;

#[must_use]
pub fn build_checks(
    orchestrator: &testing::Orchestrator,
    running: &[testing::RunningPeer],
) -> Vec<testing::ReportCheck> {
    let mut checks: Vec<testing::ReportCheck> = Vec::new();
    for (index, slot) in running.iter().enumerate() {
        let name = orchestrator.peers.get(index).map_or_else(
            || format!("p{}", index.saturating_add(1)),
            |peer| peer.name.clone(),
        );
        if slot.child.is_none() && slot.exit.is_none() {
            checks.push(testing::ReportCheck {
                name,
                ok: false,
                detail: "never started".to_string(),
            });
            continue;
        }
        let text = std::fs::read_to_string(&slot.log).unwrap_or_default();
        let failed = text.contains(testing::SCRIPT_FAILED_MARKER);
        let done = text.contains(testing::SCRIPT_DONE_MARKER);
        let last_is_stay = orchestrator
            .peers
            .get(index)
            .is_some_and(|peer| peer.steps.last().is_some_and(|step| step.text == "stay"));
        let last_is_crash = orchestrator.peers.get(index).is_some_and(|peer| {
            peer.steps
                .last()
                .is_some_and(|step| step.text == "crash-app")
        });
        let crashed = text.contains("script crash-app");
        let ok = if last_is_stay {
            !failed && slot.exit.is_none()
        } else if last_is_crash {
            !failed && slot.exit.is_some() && crashed
        } else {
            !failed && matches!(slot.exit, Some(testing::PeerExit::Code(0))) && done
        };
        checks.push(testing::ReportCheck {
            name,
            ok,
            detail: format!("exit={:?} failed={failed} done={done}", slot.exit),
        });
    }
    checks
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(super::build_checks);
    }
}
