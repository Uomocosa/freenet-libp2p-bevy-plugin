use crate::p2p;
use crate::room_sync;

pub fn drain(events: &room_sync::bevy_systems::NetEvents) -> Vec<p2p::NetEvent> {
    let Ok(mut receiver) = events.receiver.lock() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    loop {
        match receiver.try_recv() {
            Ok(event) => out.push(event),
            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(skipped)) => {
                tracing::warn!(target: "room_sync", "net events lagged, dropped={skipped}");
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
    use crate::room_sync::bevy_systems::NetEvents;
    use std::sync::Mutex;

    fn lagged_events() -> NetEvents {
        let (tx, rx) = tokio::sync::broadcast::channel(2);
        let first = p2p::NetEvent::PeerConnected(identity::PeerId::from("a"));
        let second = p2p::NetEvent::PeerConnected(identity::PeerId::from("b"));
        let third = p2p::NetEvent::PeerConnected(identity::PeerId::from("c"));
        let fourth = p2p::NetEvent::PeerConnected(identity::PeerId::from("d"));
        tx.send(first).ok();
        tx.send(second).ok();
        tx.send(third).ok();
        tx.send(fourth).ok();
        NetEvents {
            receiver: Mutex::new(rx),
        }
    }

    #[test]
    fn test_usage() {
        let events = lagged_events();
        let drained = drain(&events);
        assert_eq!(
            drained,
            vec![
                p2p::NetEvent::PeerConnected(identity::PeerId::from("c")),
                p2p::NetEvent::PeerConnected(identity::PeerId::from("d")),
            ]
        );
    }
}
