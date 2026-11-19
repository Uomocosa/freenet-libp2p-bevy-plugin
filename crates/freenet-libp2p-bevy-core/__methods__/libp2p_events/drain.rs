use crate::p2p;
use crate::replica_sync;

pub fn drain(events: &replica_sync::bevy_systems::Libp2pEvents) -> Vec<p2p::Libp2pEvent> {
    let Ok(mut receiver) = events.receiver.lock() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    loop {
        match receiver.try_recv() {
            Ok(event) => out.push(event),
            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(skipped)) => {
                tracing::warn!(target: "replica_sync", "libp2p events lagged, dropped={skipped}");
            }
            Err(
                tokio::sync::broadcast::error::TryRecvError::Empty
                | tokio::sync::broadcast::error::TryRecvError::Closed,
            ) => break,
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::drain;
    use crate::identity;
    use crate::p2p;
    use crate::testing;

    #[test]
    fn test_usage() {
        let events = testing::libp2p_events(
            [
                p2p::Libp2pEvent::PeerConnected(identity::PeerId::from("a")),
                p2p::Libp2pEvent::PeerConnected(identity::PeerId::from("b")),
                p2p::Libp2pEvent::PeerConnected(identity::PeerId::from("c")),
                p2p::Libp2pEvent::PeerConnected(identity::PeerId::from("d")),
            ],
            2,
        );
        let drained = drain(&events);
        assert_eq!(
            drained,
            vec![
                p2p::Libp2pEvent::PeerConnected(identity::PeerId::from("c")),
                p2p::Libp2pEvent::PeerConnected(identity::PeerId::from("d")),
            ]
        );
    }
}
