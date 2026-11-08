use crate::discovery;
use crate::identity;
use crate::testing;
use crate::time;

pub fn handle_client_frame(
    shared: &mut testing::orchestrator_shared::OrchestratorShared,
    peer: &identity::PeerId,
    frame: testing::FakeWireClientFrame,
    now: time::UnixTime,
) -> (Vec<testing::RoutedEvent>, Option<discovery::Lobby>) {
    let Some(game) = shared.peer_games.get(peer).cloned() else {
        return (Vec::new(), None);
    };
    match frame {
        testing::FakeWireClientFrame::Libp2pCommand(command) => {
            (shared.router_for(&game).route(peer, command), None)
        }
        testing::FakeWireClientFrame::HeartbeatTargets(targets) => {
            let lobby = shared.router_for(&game).refresh_lobby(peer, &targets, now);
            (Vec::new(), Some(lobby))
        }
        testing::FakeWireClientFrame::JoinedRoomBytes(data) => (
            shared.router_for(&game).route_joined_room_bytes(peer, data),
            None,
        ),
        testing::FakeWireClientFrame::SpecificRoomBytes { room, data } => (
            shared
                .router_for(&game)
                .route_specific_room_bytes(peer, &room, data),
            None,
        ),
    }
}
#[cfg(test)]
mod tests {
    use super::handle_client_frame;
    use crate::identity;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut shared = testing::orchestrator_shared::OrchestratorShared::new();
        let peer = identity::Peer {
            id: identity::PeerId::from("a"),
            addrs: Vec::new(),
        };
        let game = identity::GameId::from("game");
        shared.router_for(&game).register_peer(&peer);
        shared.peer_games.insert(peer.id.clone(), game);
        let (events, lobby) = handle_client_frame(
            &mut shared,
            &peer.id,
            testing::FakeWireClientFrame::JoinedRoomBytes(Vec::new()),
            time::UnixTime::from(1),
        );
        assert_eq!(events, Vec::new());
        assert_eq!(lobby, None);
    }
}
