use crate::testing;

#[must_use]
pub async fn prepare_peer_spawn(
    orchestrator: &testing::Orchestrator,
    index: usize,
    room: &str,
    scripts: &[String],
    token: &str,
    running: &[testing::RunningPeer],
    live: &[testing::LivePeer],
    dir: &std::path::Path,
) -> Option<(String, std::path::PathBuf)> {
    let already = running.get(index).is_some_and(|peer| peer.child.is_some());
    if already {
        return None;
    }
    if !orchestrator.runs_here(index) {
        return None;
    }
    let starts_after = orchestrator
        .peers
        .get(index)
        .and_then(|peer| peer.starts_after);
    let due = match starts_after {
        None => true,
        Some(checkpoint) => {
            let needle = format!(
                "{}{checkpoint}",
                testing::SCRIPT_CHECKPOINT_COMPLETED_PREFIX
            );
            let seen = running.iter().any(|peer| {
                std::fs::read_to_string(&peer.log)
                    .unwrap_or_default()
                    .contains(&needle)
            });
            if seen {
                true
            } else if let Some(board) = orchestrator.board.clone() {
                board.has_completed(&checkpoint).await.unwrap_or(false)
            } else {
                false
            }
        }
    };
    if !due {
        return None;
    }
    let relaunched_name = orchestrator
        .peers
        .get(index)
        .and_then(|peer| peer.starts_after_exit_of.clone());
    if let Some(name) = relaunched_name {
        let complete = running.iter().enumerate().all(|(slot, state)| {
            let is_prior = slot < index
                && orchestrator
                    .peers
                    .get(slot)
                    .is_some_and(|peer| peer.name == name);
            !is_prior || matches!(state.exit, Some(testing::PeerExit::Code(0)))
        });
        if !complete {
            return None;
        }
    }
    let peer = orchestrator.peers.get(index)?;
    let script = if matches!(orchestrator.network, testing::Network::DirectDial) {
        orchestrator.substituted_script(index, room, token, live)?
    } else {
        scripts.get(index)?.clone()
    };
    let stem = format!("instance-{}", peer.name);
    let mut log = dir.join(format!("{stem}.log"));
    let mut relaunches = 0u32;
    while log.exists() {
        relaunches = relaunches.saturating_add(1);
        log = dir.join(format!("{stem}-relaunch{relaunches}.log"));
    }
    Some((script, log))
}

#[cfg(test)]
mod tests {
    use super::prepare_peer_spawn;
    use crate::testing;

    fn orchestrator(network: testing::Network) -> testing::Orchestrator {
        testing::Orchestrator::from_game_and_network("starboard".to_string(), network)
    }

    #[tokio::test]
    async fn test_usage() {
        let mut orchestrator = orchestrator(testing::Network::Fake);
        orchestrator.peer("alice");
        let _ = testing::Orchestrator::runs(
            &mut orchestrator,
            [vec![testing::StepWithDeadline::from(
                testing::E2EStep::Stay,
            )]],
        );
        let scripts = vec![orchestrator.peer_script(0, "room-1").expect("script")];
        let dir = tempfile::tempdir().expect("tempdir");
        let prepared = prepare_peer_spawn(
            &orchestrator,
            0,
            "room-1",
            &scripts,
            "token",
            &[],
            &[],
            dir.path(),
        )
        .await
        .expect("ready peer");
        assert!(prepared.0.contains("stay"));
        assert!(prepared.1.ends_with("instance-alice.log"));
        assert_eq!(
            prepare_peer_spawn(
                &orchestrator,
                7,
                "room-1",
                &scripts,
                "token",
                &[],
                &[],
                dir.path()
            )
            .await,
            None
        );
    }

    #[tokio::test]
    async fn direct_dial_defers_until_addresses_are_known() {
        let mut orchestrator = orchestrator(testing::Network::DirectDial);
        let mut builder = testing::PeerBuilder::new("alice".to_string());
        builder.steps.push(testing::StoredStep {
            text: "dial:bob".to_string(),
            is_wait: false,
            within: None,
        });
        orchestrator.peers.push(builder);
        let dir = tempfile::tempdir().expect("tempdir");
        assert_eq!(
            prepare_peer_spawn(
                &orchestrator,
                0,
                "room-1",
                &[],
                "token",
                &[],
                &[],
                dir.path()
            )
            .await,
            None
        );
    }
}
