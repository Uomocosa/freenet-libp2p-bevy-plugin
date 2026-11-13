use tokio::sync::mpsc::UnboundedSender;

use crate::identity;
use crate::testing;

pub fn connect_peer(
    shared: &mut testing::orchestrator_shared::OrchestratorShared,
    peer: &identity::Peer,
    game: &identity::GameId,
    sender: UnboundedSender<testing::FakeWireServerFrame>,
) {
    shared.router_for(game).register_peer(peer);
    shared.add_sender(&peer.id, game, sender);
}
#[cfg(test)]
mod tests {
    use super::connect_peer;
    use crate::identity;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut shared = testing::orchestrator_shared::OrchestratorShared::default();
        let game = identity::GameId::from("game");
        let peer = identity::Peer {
            id: identity::PeerId::from("a"),
            addrs: Vec::new(),
        };
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        connect_peer(&mut shared, &peer, &game, tx);
        assert!(shared.senders.contains_key(&peer.id));
    }
}
