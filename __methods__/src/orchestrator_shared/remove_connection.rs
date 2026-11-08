use crate::identity;
use crate::testing;

pub fn remove_connection(
    shared: &mut testing::orchestrator_shared::OrchestratorShared,
    peer: &identity::PeerId,
) -> Vec<testing::RoutedEvent> {
    let Some(senders) = shared.senders.get_mut(peer) else {
        return Vec::new();
    };
    senders.pop();
    if !senders.is_empty() {
        return Vec::new();
    }
    shared.senders.remove(peer);
    let Some(game) = shared.peer_games.remove(peer) else {
        return Vec::new();
    };
    shared.router_for(&game).remove_peer(peer)
}

#[cfg(test)]
mod tests {
    use super::remove_connection;
    use crate::identity;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut shared = testing::orchestrator_shared::OrchestratorShared::new();
        let peer = identity::Peer {
            id: identity::PeerId::from("a"),
            addrs: Vec::new(),
        };
        let game = identity::GameId::from("game");
        shared.router_for(&game).register_peer(&peer);
        let (first_tx, _first_rx) = tokio::sync::mpsc::unbounded_channel();
        let (second_tx, _second_rx) = tokio::sync::mpsc::unbounded_channel();
        shared.add_sender(&peer.id, &game, first_tx);
        shared.add_sender(&peer.id, &game, second_tx);
        let events = remove_connection(&mut shared, &peer.id);
        assert_eq!(events, Vec::new());
        assert!(shared.peer_games.contains_key(&peer.id));
        let events = remove_connection(&mut shared, &peer.id);
        assert_eq!(events, Vec::new());
        assert!(!shared.peer_games.contains_key(&peer.id));
        let events = remove_connection(&mut shared, &peer.id);
        assert_eq!(events, Vec::new());
    }
}
