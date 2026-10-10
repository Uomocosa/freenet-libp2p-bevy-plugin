use libp2p::gossipsub;
use libp2p::kad;
use libp2p::swarm::dial_opts::PeerCondition;

use crate::net_id;
use crate::p2p;

pub fn dispatch_net_command<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    room_queries: &mut std::collections::HashMap<libp2p::kad::QueryId, net_id::RoomName>,
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
        p2p::NetCommand::FetchRoster { room } => {
            let _ = swarm
                .behaviour_mut()
                .kademlia
                .start_providing(p2p::provider_key(&room));
            let id = swarm
                .behaviour_mut()
                .kademlia
                .get_providers(p2p::provider_key(&room));
            room_queries.insert(id, room);
        }
        p2p::NetCommand::Subscribe { topic } => {
            let topic = gossipsub::IdentTopic::new(topic.as_str());
            let _ = swarm.behaviour_mut().gossipsub.subscribe(&topic);
        }
        p2p::NetCommand::Publish { topic, data } => {
            let topic = gossipsub::IdentTopic::new(topic.as_str());
            let _ = swarm.behaviour_mut().gossipsub.publish(topic, data);
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

    use libp2p::gossipsub::IdentTopic;
    use libp2p::identity::Keypair;

    use super::dispatch_net_command;
    use crate::net_id;
    use crate::p2p;

    fn swarm() -> libp2p::Swarm<p2p::Behaviour<u32>> {
        p2p::build_swarm::<u32>(Keypair::generate_ed25519(), p2p::MdnsMode::Disabled).unwrap()
    }

    #[tokio::test]
    async fn test_usage() {
        let mut swarm = swarm();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut room_queries = HashMap::new();
        let topic = net_id::Topic::from("lobby/topic");
        dispatch_net_command(
            &mut swarm,
            &tx,
            &mut room_queries,
            p2p::NetCommand::Subscribe {
                topic: topic.clone(),
            },
        );
        let hash = IdentTopic::new(topic.as_str()).hash();
        assert!(swarm.behaviour().gossipsub.topics().any(|t| *t == hash));

        let room = net_id::RoomName::from("room-a");
        dispatch_net_command(
            &mut swarm,
            &tx,
            &mut room_queries,
            p2p::NetCommand::FindRoom { room: room.clone() },
        );
        assert_eq!(room_queries.values().collect::<Vec<_>>(), vec![&room]);
    }
}
