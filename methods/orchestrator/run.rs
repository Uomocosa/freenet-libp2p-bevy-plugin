use std::time::Instant;

use crate::testing;

pub async fn run(orchestrator: &mut testing::Orchestrator) -> testing::OrchestratorReport {
    let game = orchestrator.game.clone();
    if matches!(orchestrator.network, testing::Network::LocalGateways) {
        return testing::Orchestrator::early_report(
            &game,
            std::path::PathBuf::new(),
            "local-gateways network is not implemented yet".to_string(),
        );
    }
    let (dir, room, token) = match orchestrator.setup_run_dir() {
        Ok(parts) => parts,
        Err(detail) => {
            return testing::Orchestrator::early_report(
                &game,
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".local-run"),
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
    let started = Instant::now();
    let cap = orchestrator.cap();
    orchestrator
        .drive_until_done(
            &mut running,
            &dir,
            &bin,
            &scripts,
            &token,
            &transport,
            orchestrator_port,
            cap,
            started,
        )
        .await;
    let checks = orchestrator.build_checks(&running);
    testing::Orchestrator::finalize_run(&game, &room, &transport, dir, &mut running, checks)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(super::run);
    }
}
