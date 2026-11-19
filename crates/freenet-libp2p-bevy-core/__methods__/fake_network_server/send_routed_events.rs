use std::sync::Arc;

use crate::testing;

pub async fn send_routed_events(
    task: &testing::FakeNetworkServer,
    shared: &Arc<tokio::sync::Mutex<testing::orchestrator_shared::OrchestratorShared>>,
    events: Vec<testing::RoutedEvent>,
) {
    let _ = *task;
    for event in events {
        let senders = {
            let guard = shared.lock().await;
            guard.senders.get(&event.to).cloned().unwrap_or_default()
        };
        for sender in senders {
            sender
                .send(testing::FakeWireServerFrame::Libp2pEvent(
                    event.event.clone(),
                ))
                .ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::send_routed_events;
    use crate::identity;
    use crate::p2p;
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let task = testing::FakeNetworkServer::start().await.expect("server");
        let peer = identity::PeerId::from("peer");
        let (sender, mut frames) = tokio::sync::mpsc::unbounded_channel();
        let mut shared = testing::orchestrator_shared::OrchestratorShared::default();
        shared.add_sender(&peer, &identity::GameId::from("game"), sender);
        let shared = Arc::new(tokio::sync::Mutex::new(shared));
        let routed = {
            send_routed_events(
                &task,
                &shared,
                vec![testing::RoutedEvent {
                    to: peer.clone(),
                    event: p2p::Libp2pEvent::PeerConnected(peer.clone()),
                }],
            )
            .await;
            frames.try_recv()
        };
        assert_eq!(
            routed,
            Ok(testing::FakeWireServerFrame::Libp2pEvent(
                p2p::Libp2pEvent::PeerConnected(peer)
            ))
        );
    }
}
