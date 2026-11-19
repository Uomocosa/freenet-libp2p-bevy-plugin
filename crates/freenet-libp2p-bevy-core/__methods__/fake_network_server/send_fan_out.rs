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
    use std::sync::Arc;

    use super::send_fan_out;
    use crate::identity;
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let task = testing::FakeNetworkServer::start().await.expect("server");
        let game = identity::GameId::from("game");
        let alice = identity::PeerId::from("alice");
        let bob = identity::PeerId::from("bob");
        let (alice_sender, mut alice_frames) = tokio::sync::mpsc::unbounded_channel();
        let (bob_sender, mut bob_frames) = tokio::sync::mpsc::unbounded_channel();
        let mut shared = testing::orchestrator_shared::OrchestratorShared::default();
        shared.add_sender(&alice, &game, alice_sender);
        shared.add_sender(&bob, &game, bob_sender);
        let shared = Arc::new(tokio::sync::Mutex::new(shared));

        let (alice_game, bob_game) = {
            send_fan_out(
                &task,
                &shared,
                &game,
                vec![testing::FanOut {
                    to: testing::FanOutTarget::Game,
                    bytes: vec![7],
                }],
            )
            .await;
            (alice_frames.try_recv(), bob_frames.try_recv())
        };
        assert_eq!(
            (alice_game, bob_game),
            (
                Ok(testing::FakeWireServerFrame::Extension(vec![7])),
                Ok(testing::FakeWireServerFrame::Extension(vec![7])),
            )
        );

        let (alice_direct, bob_direct) = {
            send_fan_out(
                &task,
                &shared,
                &game,
                vec![testing::FanOut {
                    to: testing::FanOutTarget::Peer(bob.clone()),
                    bytes: vec![9],
                }],
            )
            .await;
            (alice_frames.try_recv(), bob_frames.try_recv())
        };
        assert!(alice_direct.is_err());
        assert_eq!(
            bob_direct,
            Ok(testing::FakeWireServerFrame::Extension(vec![9]))
        );
    }
}
