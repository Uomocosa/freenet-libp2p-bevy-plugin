use crate::identity;
use crate::p2p;
use crate::testing;

pub fn route(
    router: &mut testing::PeerMessageRouter,
    from: &identity::PeerId,
    command: p2p::Libp2pCommand,
) -> Vec<testing::RoutedEvent> {
    if !router.registered_peers.contains(from) {
        return Vec::new();
    }
    match command {
        p2p::Libp2pCommand::Dial(peer) => router.connect_dial_target(from, peer),
        command @ (p2p::Libp2pCommand::Subscribe { .. }
        | p2p::Libp2pCommand::Unsubscribe { .. }
        | p2p::Libp2pCommand::SubscribeRoom { .. }
        | p2p::Libp2pCommand::UnsubscribeRoom { .. }) => {
            router.update_peer_subscriptions(from, command)
        }
        command @ (p2p::Libp2pCommand::PutHistoryRecord { .. }
        | p2p::Libp2pCommand::GetHistoryRecord { .. }
        | p2p::Libp2pCommand::MeasureHistoryStore
        | p2p::Libp2pCommand::RemoveHistoryRecords { .. }) => {
            router.serve_history_command(from, command)
        }
        command @ (p2p::Libp2pCommand::FetchSnapshot { .. }
        | p2p::Libp2pCommand::FetchReplicaSlice { .. }
        | p2p::Libp2pCommand::RespondSnapshot { .. }) => {
            router.serve_room_sync_command(from, command)
        }
        command
        @ (p2p::Libp2pCommand::Publish { .. } | p2p::Libp2pCommand::PeerExchange { .. }) => {
            router.fan_out_peer_message(from, command)
        }
        p2p::Libp2pCommand::ReserveRelay { .. }
        | p2p::Libp2pCommand::SetMdns { .. }
        | p2p::Libp2pCommand::AddKadPeer(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::route;
    use crate::identity;
    use crate::p2p;
    use crate::testing;

    fn registered(router: &mut testing::PeerMessageRouter, id: &str) -> identity::PeerId {
        let peer = identity::Peer {
            id: identity::PeerId::from(id),
            addrs: Vec::new(),
        };
        router.register_peer(&peer);
        peer.id
    }

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let dialer = registered(&mut router, "a");
        registered(&mut router, "b");
        let events = route(
            &mut router,
            &dialer,
            p2p::Libp2pCommand::Dial(identity::Peer {
                id: identity::PeerId::from("b"),
                addrs: Vec::new(),
            }),
        );
        assert_eq!(events.len(), 4);
    }

    #[test]
    fn commands_from_unregistered_peers_are_dropped() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        registered(&mut router, "b");
        let events = route(
            &mut router,
            &identity::PeerId::from("ghost"),
            p2p::Libp2pCommand::Dial(identity::Peer {
                id: identity::PeerId::from("b"),
                addrs: Vec::new(),
            }),
        );
        assert_eq!(events, Vec::new());
        let events = route(
            &mut router,
            &identity::PeerId::from("ghost"),
            p2p::Libp2pCommand::MeasureHistoryStore,
        );
        assert_eq!(events, Vec::new());
    }

    #[test]
    fn relay_mdns_and_kad_commands_have_no_effect() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let dialer = registered(&mut router, "a");
        let relay = identity::Peer {
            id: identity::PeerId::from("relay"),
            addrs: vec![identity::PeerAddr::from("/ip4/1/tcp/4001")],
        };
        let commands = [
            p2p::Libp2pCommand::ReserveRelay {
                relay_addr: identity::PeerAddr::from("/ip4/1/tcp/4001"),
            },
            p2p::Libp2pCommand::SetMdns {
                mode: p2p::MdnsMode::Disabled,
            },
            p2p::Libp2pCommand::AddKadPeer(relay),
        ];
        for command in commands {
            assert_eq!(route(&mut router, &dialer, command), Vec::new());
        }
    }
}
