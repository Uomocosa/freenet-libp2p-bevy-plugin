use crate::testing;

pub async fn run(orchestrator: &mut testing::Orchestrator) -> testing::OrchestratorReport {
    let game = orchestrator.game.clone();
    if testing::Orchestrator::is_check_role() {
        let dir = match testing::Orchestrator::check_results_dir() {
            Ok(dir) => dir,
            Err(detail) => {
                return testing::Orchestrator::early_report(
                    &game,
                    std::path::PathBuf::new(),
                    detail,
                );
            }
        };
        return orchestrator.run_check_role(&dir).await;
    }
    if matches!(orchestrator.network, testing::Network::LocalGateways) {
        return testing::Orchestrator::early_report(
            &game,
            std::path::PathBuf::new(),
            "local-gateways network is not implemented yet".to_string(),
        );
    }
    let raw_placement = std::env::var("E2E_PLACEMENT").unwrap_or_default();
    let placement = match testing::Orchestrator::parse_machine_placement(&raw_placement) {
        Ok(placement) => placement,
        Err(detail) => {
            return testing::Orchestrator::early_report(
                &game,
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../..")
                    .join(".local-run"),
                detail,
            );
        }
    };
    if !placement.is_empty() {
        let machine = testing::Orchestrator::this_machine();
        let local = orchestrator
            .peers
            .iter()
            .filter(|peer| testing::Orchestrator::peer_runs_here(&peer.name, &machine, &placement))
            .count();
        if local == 0 {
            return testing::Orchestrator::early_report(
                &game,
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../..")
                    .join(".local-run"),
                format!("no peers placed on machine '{machine}'"),
            );
        }
        orchestrator.machine = machine;
    }
    orchestrator.placement = placement;
    let (dir, room, token) = match orchestrator.setup_run_dir() {
        Ok(parts) => parts,
        Err(detail) => {
            return testing::Orchestrator::early_report(
                &game,
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../..")
                    .join(".local-run"),
                detail,
            );
        }
    };
    let (bin, orchestrator_port, scripts) = match orchestrator.setup_bin_port_scripts(&room).await {
        Ok(parts) => parts,
        Err(detail) => {
            return testing::Orchestrator::early_report(&game, dir, detail);
        }
    };
    let transport = std::env::var("LOBBY_TRANSPORT").unwrap_or_else(|_| "both".to_string());
    let mut running: Vec<testing::RunningPeer> = orchestrator
        .peers
        .iter()
        .map(|peer| testing::RunningPeer {
            log: dir.join(format!("instance-{}.log", peer.name)),
            child: None,
            exit: None,
        })
        .collect();
    orchestrator
        .drive_until_done(
            &mut running,
            &dir,
            &bin,
            &room,
            &scripts,
            &token,
            &transport,
            orchestrator_port,
        )
        .await;
    let checks = orchestrator.build_checks(&running, &token).await;
    testing::Orchestrator::finalize_run(&game, &room, &transport, dir, &mut running, checks)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(super::run);
    }
}
