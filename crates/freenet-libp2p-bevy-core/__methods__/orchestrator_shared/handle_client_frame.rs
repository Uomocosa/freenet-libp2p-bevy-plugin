use crate::identity;
use crate::testing;

pub fn handle_client_frame(
    shared: &mut testing::orchestrator_shared::OrchestratorShared,
    peer: &identity::PeerId,
    frame: testing::FakeWireClientFrame,
) -> Vec<testing::RoutedEvent> {
    let Some(game) = shared.peer_games.get(peer).cloned() else {
        return Vec::new();
    };
    match frame {
        testing::FakeWireClientFrame::Libp2pCommand(command) => {
            shared.router_for(&game).route(peer, command)
        }
        testing::FakeWireClientFrame::Extension(_) => Vec::new(),
        testing::FakeWireClientFrame::JoinedZoneBytes(data) => {
            shared.router_for(&game).route_joined_zone_bytes(peer, data)
        }
        testing::FakeWireClientFrame::SpecificZoneBytes { zone, data } => shared
            .router_for(&game)
            .route_specific_zone_bytes(peer, &zone, data),
    }
}
#[cfg(test)]
mod tests {
    use super::handle_client_frame;
    use crate::identity;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut shared = testing::orchestrator_shared::OrchestratorShared::default();
        let peer = identity::Peer {
            id: identity::PeerId::from("a"),
            addrs: Vec::new(),
        };
        let game = identity::GameId::from("game");
        shared.router_for(&game).register_peer(&peer);
        shared.peer_games.insert(peer.id.clone(), game);
        let events = handle_client_frame(
            &mut shared,
            &peer.id,
            testing::FakeWireClientFrame::JoinedZoneBytes(Vec::new()),
        );
        assert_eq!(events, Vec::new());
    }
}
