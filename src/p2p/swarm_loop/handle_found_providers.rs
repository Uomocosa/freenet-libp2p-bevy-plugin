use crate::net_id;
use crate::p2p;

pub fn handle_found_providers<T: p2p::Message>(
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    room_queries: &std::collections::HashMap<libp2p::kad::QueryId, net_id::RoomName>,
    id: libp2p::kad::QueryId,
    providers: &std::collections::HashSet<libp2p::PeerId>,
) {
    if let Some(room) = room_queries.get(&id) {
        let peers = providers
            .iter()
            .map(|peer| net_id::PeerId(peer.to_string()))
            .collect::<Vec<_>>();
        event_tx
            .send(p2p::Event::Net(p2p::NetEvent::RoomProviders {
                room: room.clone(),
                peers,
            }))
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use libp2p::identity::Keypair;

    use super::handle_found_providers;
    use crate::net_id;
    use crate::p2p;

    fn swarm() -> libp2p::Swarm<p2p::Behaviour<u32>> {
        p2p::build_swarm::<u32>(Keypair::generate_ed25519(), p2p::MdnsMode::Disabled).unwrap()
    }

    #[tokio::test]
    async fn test_usage() {
        let mut swarm = swarm();
        let id = swarm
            .behaviour_mut()
            .kademlia
            .get_providers(p2p::provider_key(&net_id::RoomName::from("room-a")));
        let room_queries = HashMap::from([(id, net_id::RoomName::from("room-a"))]);
        let provider = libp2p::PeerId::random();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event<u32>>();
        handle_found_providers(&tx, &room_queries, id, &HashSet::from([provider]));
        let Ok(p2p::Event::Net(p2p::NetEvent::RoomProviders { room, peers })) = rx.try_recv()
        else {
            panic!("expected room providers");
        };
        assert_eq!(room.as_str(), "room-a");
        assert_eq!(peers, vec![net_id::PeerId(provider.to_string())]);
    }
}
