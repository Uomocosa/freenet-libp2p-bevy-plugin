use crate::net_id;
use crate::p2p;

pub fn note_dial_failure<T: p2p::Message>(
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    peer_id: Option<libp2p::PeerId>,
    error: &libp2p::swarm::DialError,
) {
    match peer_id {
        Some(peer_id) => {
            event_tx
                .send(p2p::Event::Net(p2p::NetEvent::DialFailed {
                    peer_id: net_id::PeerId(peer_id.to_string()),
                    reason: error.to_string(),
                }))
                .ok();
        }
        None => {
            tracing::debug!(target: "p2p", error = %error, "p2p dial failed without peer");
        }
    }
}

#[cfg(test)]
mod tests {
    use libp2p::swarm::DialError;

    use super::note_dial_failure;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event<u32>>();
        let peer = libp2p::PeerId::random();
        note_dial_failure(&tx, Some(peer), &DialError::NoAddresses);
        let Ok(p2p::Event::Net(p2p::NetEvent::DialFailed { peer_id, .. })) = rx.try_recv() else {
            panic!("expected a dial failure");
        };
        assert_eq!(peer_id.as_str(), peer.to_string());
        note_dial_failure(&tx, None, &DialError::NoAddresses);
        assert!(rx.try_recv().is_err());
    }
}
