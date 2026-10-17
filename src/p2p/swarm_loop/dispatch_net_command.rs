use libp2p::gossipsub;
use libp2p::kad;
use libp2p::swarm::dial_opts::PeerCondition;

use crate::identity;
use crate::p2p;

pub fn dispatch_net_command<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    room_queries: &mut std::collections::HashMap<libp2p::kad::QueryId, identity::RoomName>,
    rooms: &mut p2p::swarm_loop::RoomTopics,
    command: p2p::NetCommand,
) {
    match command {
        p2p::NetCommand::Dial(peer) => {
            p2p::swarm_loop::dial_peer(
                swarm,
                event_tx,
                &peer.id,
                &peer.addrs,
                PeerCondition::DisconnectedAndNotDialing,
            );
        }
        p2p::NetCommand::ReserveRelay { relay_addr } => {
            if let Ok(addr) = relay_addr.parse::<libp2p::Multiaddr>() {
                let _ = swarm.listen_on(addr);
            }
        }
        p2p::NetCommand::SetMdns { mode } => {
            let peer_id = *swarm.local_peer_id();
            swarm.behaviour_mut().mdns = p2p::mdns_behaviour(mode, peer_id);
        }
        p2p::NetCommand::AddKadPeer(peer) => {
            p2p::swarm_loop::seed_kad_peer(swarm, &peer.id, &peer.addrs);
        }
        p2p::NetCommand::ProvideRoom { room } => {
            let _ = swarm
                .behaviour_mut()
                .kademlia
                .start_providing(p2p::provider_key(&room));
        }
        p2p::NetCommand::FindRoom { room } => {
            let id = swarm
                .behaviour_mut()
                .kademlia
                .get_providers(p2p::provider_key(&room));
            room_queries.insert(id, room);
        }
        p2p::NetCommand::PutHistory { id, data } => {
            let record_key = p2p::record_key(&id);
            let record = kad::Record {
                key: record_key.clone(),
                value: data,
                publisher: None,
                expires: None,
            };
            let _ = swarm
                .behaviour_mut()
                .kademlia
                .put_record(record, kad::Quorum::One);
            let _ = swarm.behaviour_mut().kademlia.start_providing(record_key);
        }
        p2p::NetCommand::FetchHistory { id } => {
            swarm
                .behaviour_mut()
                .kademlia
                .get_record(p2p::record_key(&id));
        }
        p2p::NetCommand::Subscribe { topic } => {
            let topic = gossipsub::IdentTopic::new(topic.as_str());
            let _ = swarm.behaviour_mut().gossipsub.subscribe(&topic);
        }
        p2p::NetCommand::SubscribeRoom { room } => {
            let topic = gossipsub::IdentTopic::new(p2p::room_topic(&rooms.game, &room).as_str());
            match swarm.behaviour_mut().gossipsub.subscribe(&topic) {
                Ok(_) => {
                    tracing::info!(target: "p2p", room = %room.as_str(), "p2p room subscribed");
                    rooms.register(&topic, room);
                }
                Err(error) => {
                    event_tx
                        .send(p2p::Event::Error(format!(
                            "room subscribe {}: {error}",
                            room.as_str()
                        )))
                        .ok();
                }
            }
        }
        p2p::NetCommand::UnsubscribeRoom { room } => {
            let topic = gossipsub::IdentTopic::new(p2p::room_topic(&rooms.game, &room).as_str());
            let _ = swarm.behaviour_mut().gossipsub.unsubscribe(&topic);
            rooms.unregister(&topic);
            tracing::info!(target: "p2p", room = %room.as_str(), "p2p room unsubscribed");
        }
        p2p::NetCommand::Publish { topic, data } => {
            let topic = gossipsub::IdentTopic::new(topic.as_str());
            if let Err(error) = swarm.behaviour_mut().gossipsub.publish(topic, data) {
                event_tx
                    .send(p2p::Event::Error(format!("gossip publish: {error}")))
                    .ok();
            }
        }
        p2p::NetCommand::Exchange { peer_id, data } => {
            if let Ok(pid) = peer_id.parse::<libp2p::PeerId>() {
                swarm.behaviour_mut().exchange.send_request(&pid, data);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use libp2p::gossipsub;
    use libp2p::gossipsub::IdentTopic;
    use libp2p::identity::Keypair;

    use super::dispatch_net_command;
    use crate::identity;
    use crate::p2p;

    fn swarm() -> libp2p::Swarm<p2p::Behaviour<u32>> {
        p2p::build_swarm::<u32>(
            Keypair::generate_ed25519(),
            p2p::MdnsMode::Disabled,
            &p2p::GossipsubConfig::default(),
        )
        .unwrap()
    }

    fn rooms() -> p2p::swarm_loop::RoomTopics {
        p2p::swarm_loop::RoomTopics::new(identity::GameId::from("game"))
    }

    #[tokio::test]
    async fn test_usage() {
        let mut swarm = swarm();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut room_queries = HashMap::new();
        let mut rooms = rooms();
        let topic = identity::Topic::from("lobby/topic");
        dispatch_net_command(
            &mut swarm,
            &tx,
            &mut room_queries,
            &mut rooms,
            p2p::NetCommand::Subscribe {
                topic: topic.clone(),
            },
        );
        let hash = IdentTopic::new(topic.as_str()).hash();
        assert!(swarm.behaviour().gossipsub.topics().any(|t| *t == hash));

        let room = identity::RoomName::from("room-a");
        dispatch_net_command(
            &mut swarm,
            &tx,
            &mut room_queries,
            &mut rooms,
            p2p::NetCommand::FindRoom { room: room.clone() },
        );
        assert_eq!(room_queries.values().collect::<Vec<_>>(), vec![&room]);
    }

    #[tokio::test]
    async fn room_subscribe_and_unsubscribe_track_the_map() {
        let mut swarm = swarm();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut room_queries = HashMap::new();
        let mut rooms = rooms();
        let room = identity::RoomName::from("room-a");
        let hash = gossipsub::IdentTopic::new(p2p::room_topic(&rooms.game, &room).as_str()).hash();
        dispatch_net_command(
            &mut swarm,
            &tx,
            &mut room_queries,
            &mut rooms,
            p2p::NetCommand::SubscribeRoom { room: room.clone() },
        );
        assert_eq!(rooms.room(&hash), Some(&room));
        assert!(swarm.behaviour().gossipsub.topics().any(|t| *t == hash));
        dispatch_net_command(
            &mut swarm,
            &tx,
            &mut room_queries,
            &mut rooms,
            p2p::NetCommand::UnsubscribeRoom { room },
        );
        assert!(rooms.room(&hash).is_none());
        assert!(!swarm.behaviour().gossipsub.topics().any(|t| *t == hash));
    }
}
