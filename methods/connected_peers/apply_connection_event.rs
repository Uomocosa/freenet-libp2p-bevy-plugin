use crate::direct_dial;
use crate::p2p;

pub fn apply_connection_event(
    peers: &mut direct_dial::ConnectedPeers,
    event: &p2p::Libp2pEvent,
) -> bool {
    let mut changed = false;
    match event {
        p2p::Libp2pEvent::PeerConnected(id)
        | p2p::Libp2pEvent::Libp2pConnectionEstablished { peer_id: id, .. } => {
            changed = peers.insert(id.clone());
        }
        p2p::Libp2pEvent::PeerDisconnected(id) => {
            changed = peers.remove(id);
        }
        _ => {}
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::apply_connection_event;
    use crate::direct_dial;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let mut peers = direct_dial::ConnectedPeers::default();
        let id = identity::PeerId::from("alice");
        assert!(apply_connection_event(
            &mut peers,
            &p2p::Libp2pEvent::PeerConnected(id.clone())
        ));
        assert!(!apply_connection_event(
            &mut peers,
            &p2p::Libp2pEvent::PeerConnected(id.clone())
        ));
        assert!(apply_connection_event(
            &mut peers,
            &p2p::Libp2pEvent::PeerDisconnected(id.clone())
        ));
        assert!(!peers.contains(&id));
    }
}
