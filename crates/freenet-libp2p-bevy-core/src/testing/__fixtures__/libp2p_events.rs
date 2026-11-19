use std::sync::Mutex;

use crate::p2p;
use crate::replica_sync;

#[must_use]
pub fn libp2p_events(
    events: impl IntoIterator<Item = p2p::Libp2pEvent>,
    capacity: usize,
) -> replica_sync::bevy_systems::Libp2pEvents {
    let (sender, receiver) = tokio::sync::broadcast::channel(capacity);
    for event in events {
        sender.send(event).ok();
    }
    replica_sync::bevy_systems::Libp2pEvents {
        receiver: Mutex::new(receiver),
    }
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::p2p;

    use super::libp2p_events;

    #[test]
    fn test_usage() {
        let first = p2p::Libp2pEvent::PeerConnected(identity::PeerId::from("a"));
        let second = p2p::Libp2pEvent::PeerConnected(identity::PeerId::from("b"));
        let events = libp2p_events([first.clone(), second.clone()], 4);
        assert_eq!(events.drain(), vec![first, second]);
    }
}
