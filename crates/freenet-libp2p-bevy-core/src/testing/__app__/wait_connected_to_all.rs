use std::time::Duration;

use crate::identity;
use crate::p2p;
use crate::testing;

pub async fn wait_connected_to_all<Payload: p2p::traits::Message>(
    node: &mut testing::Node<Payload>,
    peers: &[identity::PeerId],
) -> bool {
    let Some(deadline) = std::time::Instant::now().checked_add(Duration::from_secs(10)) else {
        return false;
    };
    let mut seen: Vec<identity::PeerId> = Vec::new();
    while seen.len() < peers.len() {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        let event = testing::wait_for_message(&mut node.events, remaining, |event| {
            matches!(
                event,
                p2p::Event::Libp2p(p2p::Libp2pEvent::PeerConnected(id)) if peers.contains(id)
            )
        })
        .await;
        let Some(p2p::Event::Libp2p(p2p::Libp2pEvent::PeerConnected(id))) = event else {
            return false;
        };
        if !seen.contains(&id) {
            seen.push(id);
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::p2p;
    use crate::testing;

    use super::wait_connected_to_all;

    #[tokio::test]
    async fn test_usage() {
        let (commands, _command_rx) = tokio::sync::mpsc::unbounded_channel();
        let (libp2p_commands, _libp2p_rx) = tokio::sync::mpsc::unbounded_channel();
        let (event_tx, events) = tokio::sync::mpsc::unbounded_channel();
        let handle = tokio::spawn(async {});
        let mut node = testing::Node::<testing::DummyMessage> {
            commands,
            libp2p_commands,
            events,
            handle,
        };
        event_tx
            .send(p2p::Event::Libp2p(p2p::Libp2pEvent::PeerConnected(
                identity::PeerId::from("a"),
            )))
            .expect("send");
        assert!(wait_connected_to_all(&mut node, &[identity::PeerId::from("a")]).await);
        node.handle.abort();
    }
}
