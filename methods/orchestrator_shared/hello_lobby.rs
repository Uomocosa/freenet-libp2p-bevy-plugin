use tokio::sync::mpsc::UnboundedSender;

use crate::discovery;
use crate::identity;
use crate::testing;

pub fn hello_lobby(
    shared: &mut testing::orchestrator_shared::OrchestratorShared,
    peer: &identity::Peer,
    game: &identity::GameId,
    sender: UnboundedSender<testing::FakeWireServerFrame>,
) -> discovery::Lobby {
    shared.router_for(game).register_peer(peer);
    shared.add_sender(&peer.id, game, sender);
    shared.router_for(game).shared_lobby.clone()
}

#[cfg(test)]
mod tests {
    use super::hello_lobby;
    use crate::discovery;
    use crate::identity;
    use crate::testing;

    fn heartbeat(room: &str) -> discovery::state_machine::HeartbeatTarget {
        discovery::state_machine::HeartbeatTarget {
            room: identity::RoomName::from(room),
            created_at: discovery::UnixTime::from(10),
            addrs: vec![identity::PeerAddr::from("/ip4/1/tcp/4001")],
            display_name: identity::Username::from("someone"),
            membership: discovery::RoomMembership::Occupant,
        }
    }

    #[test]
    fn test_usage() {
        let mut shared = testing::orchestrator_shared::OrchestratorShared::new();
        let game = identity::GameId::from("game");
        let peer = identity::Peer {
            id: identity::PeerId::from("a"),
            addrs: Vec::new(),
        };
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let lobby = hello_lobby(&mut shared, &peer, &game, tx);
        assert!(lobby.is_empty());
    }

    #[test]
    fn peer_connecting_after_a_room_was_created_sees_it_with_no_further_heartbeat() {
        let mut shared = testing::orchestrator_shared::OrchestratorShared::new();
        let game = identity::GameId::from("game");
        let first = identity::Peer {
            id: identity::PeerId::from("a"),
            addrs: Vec::new(),
        };
        shared.router_for(&game).register_peer(&first);
        let (first_tx, _first_rx) = tokio::sync::mpsc::unbounded_channel();
        shared.add_sender(&first.id, &game, first_tx);
        shared.router_for(&game).refresh_lobby(
            &first.id,
            &[heartbeat("room-a")],
            discovery::UnixTime::from(20),
        );
        let late = identity::Peer {
            id: identity::PeerId::from("b"),
            addrs: Vec::new(),
        };
        let (late_tx, _late_rx) = tokio::sync::mpsc::unbounded_channel();
        let lobby = hello_lobby(&mut shared, &late, &game, late_tx);
        assert!(
            lobby.contains_key(&identity::RoomName::from("room-a")),
            "late joiner must see the room with no heartbeat of its own"
        );
    }
}
