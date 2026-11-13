use crate::identity;
use crate::testing;
use crate::time;

pub fn remove_connection(
    shared: &mut testing::orchestrator_shared::OrchestratorShared,
    peer: &identity::PeerId,
    now: time::UnixTime,
) -> (Vec<testing::RoutedEvent>, Vec<testing::FanOut>) {
    let Some(senders) = shared.senders.get_mut(peer) else {
        return (Vec::new(), Vec::new());
    };
    senders.pop();
    if !senders.is_empty() {
        return (Vec::new(), Vec::new());
    }
    shared.senders.remove(peer);
    let Some(game) = shared.peer_games.remove(peer) else {
        return (Vec::new(), Vec::new());
    };
    let fan_outs = shared.kit.on_disconnect(peer, &game, now);
    (shared.router_for(&game).remove_peer(peer), fan_outs)
}

#[cfg(test)]
mod tests {
    use super::remove_connection;
    use crate::identity;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut shared = testing::orchestrator_shared::OrchestratorShared::default();
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
        let (events, _) = remove_connection(&mut shared, &peer.id, time::UnixTime::from_secs(1));
        assert_eq!(events, Vec::new());
        assert!(shared.peer_games.contains_key(&peer.id));
        let (events, _) = remove_connection(&mut shared, &peer.id, time::UnixTime::from_secs(1));
        assert_eq!(events, Vec::new());
        assert!(!shared.peer_games.contains_key(&peer.id));
        let (events, _) = remove_connection(&mut shared, &peer.id, time::UnixTime::from_secs(1));
        assert_eq!(events, Vec::new());
    }
}
