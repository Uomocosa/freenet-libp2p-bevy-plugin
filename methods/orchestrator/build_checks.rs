use crate::testing;

#[must_use]
pub async fn build_checks(
    orchestrator: &testing::Orchestrator,
    running: &[testing::RunningPeer],
    token: &str,
) -> Vec<testing::ReportCheck> {
    let mut checks: Vec<testing::ReportCheck> = Vec::new();
    for (index, slot) in running.iter().enumerate() {
        if !orchestrator.runs_here(index) {
            continue;
        }
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
        if orchestrator.should_forbid_freenet_node {
            let started = text.contains("lobby freenet node started");
            checks.push(testing::ReportCheck {
                name: format!("{name} no freenet node"),
                ok: !started,
                detail: if started {
                    "the embedded freenet node started".to_string()
                } else {
                    "no freenet node started".to_string()
                },
            });
        }
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
    for expected in &orchestrator.book_kind_expectations {
        if !orchestrator.placement.is_empty()
            && !testing::Orchestrator::peer_runs_here(
                &expected.owner,
                &orchestrator.machine,
                &orchestrator.placement,
            )
        {
            continue;
        }
        checks.push(orchestrator.check_book_kind(token, expected).await);
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
