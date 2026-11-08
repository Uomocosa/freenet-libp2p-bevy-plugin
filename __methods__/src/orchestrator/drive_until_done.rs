use std::path::Path;
use std::time::{Duration, Instant};

use crate::testing;

pub async fn drive_until_done(
    orchestrator: &mut testing::Orchestrator,
    running: &mut [testing::RunningPeer],
    dir: &Path,
    bin: &Path,
    room: &str,
    scripts: &[String],
    token: &str,
    transport: &str,
    orchestrator_port: Option<u16>,
) {
    const POLL: Duration = Duration::from_millis(500);
    let cap = orchestrator.cap();
    let started = Instant::now();
    loop {
        orchestrator
            .spawn_due_peers(
                running,
                dir,
                bin,
                room,
                scripts,
                token,
                transport,
                orchestrator_port,
            )
            .await;
        testing::Orchestrator::poll_exits(running);
        orchestrator.publish_ready_peers(running, token).await;
        let failed = running.iter().any(|peer| {
            let text = std::fs::read_to_string(&peer.log).unwrap_or_default();
            text.contains(testing::SCRIPT_FAILED_MARKER)
                || matches!(peer.exit, Some(testing::PeerExit::Code(code)) if code != 0)
        });
        let done = !running.is_empty()
            && running.iter().enumerate().all(|(index, peer)| {
                !orchestrator.runs_here(index) || {
                    let started = peer.child.is_some() || peer.exit.is_some();
                    let stays = orchestrator.peers.get(index).is_some_and(|builder| {
                        builder.steps.last().is_some_and(|step| step.text == "stay")
                    });
                    started && (stays || peer.exit.is_some())
                }
            });
        if failed || done || started.elapsed() >= cap {
            break;
        }
        tokio::time::sleep(POLL).await;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(super::drive_until_done);
    }
}
