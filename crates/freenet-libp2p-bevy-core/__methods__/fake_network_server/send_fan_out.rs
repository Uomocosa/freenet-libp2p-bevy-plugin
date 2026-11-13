use std::sync::Arc;

use crate::identity;
use crate::testing;

pub async fn send_fan_out(
    task: &testing::FakeNetworkServer,
    shared: &Arc<tokio::sync::Mutex<testing::orchestrator_shared::OrchestratorShared>>,
    game: &identity::GameId,
    fan_outs: Vec<testing::FanOut>,
) {
    let _ = *task;
    for fan_out in fan_outs {
        let targets = {
            let guard = shared.lock().await;
            match fan_out.to {
                testing::FanOutTarget::Peer(peer) => vec![peer],
                testing::FanOutTarget::Game => guard
                    .peer_games
                    .iter()
                    .filter(|(_, peer_game)| *peer_game == game)
                    .map(|(peer, _)| peer.clone())
                    .collect(),
            }
        };
        for peer in targets {
            let senders = {
                let guard = shared.lock().await;
                guard.senders.get(&peer).cloned().unwrap_or_default()
            };
            for sender in senders {
                sender
                    .send(testing::FakeWireServerFrame::Extension(
                        fan_out.bytes.clone(),
                    ))
                    .ok();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_usage() {
        let _ = stringify!(send_fan_out);
    }
}
