use libp2p::gossipsub;
use libp2p::swarm::dial_opts::PeerCondition;

use crate::p2p;

pub fn dispatch_net_command<Payload: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<Payload>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
    subscriptions: &mut p2p::swarm_loop::RoomSubscriptions,
    snapshot_replies: &mut p2p::swarm_loop::PendingSnapshotReplies,
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
        p2p::NetCommand::PutHistoryRecord { id, value, ttl } => {
            p2p::swarm_loop::put_history_record(swarm, event_tx, &id, value, ttl);
        }
        p2p::NetCommand::GetHistoryRecord { id } => {
            let key = p2p::history_record_key(&id);
            swarm.behaviour_mut().kademlia.get_record(key);
        }
        p2p::NetCommand::MeasureHistoryStore => {
            let bytes_by_prefix = p2p::swarm_loop::measure_history_store(swarm);
            let measured = p2p::NetEvent::HistoryStoreMeasured { bytes_by_prefix };
            event_tx.send(p2p::Event::Net(measured)).ok();
        }
        p2p::NetCommand::RemoveHistoryRecords { prefix } => {
            p2p::swarm_loop::remove_history_records(swarm, &prefix);
        }
        p2p::NetCommand::Subscribe { topic } => {
            let topic = gossipsub::IdentTopic::new(topic.as_str());
            let _ = swarm.behaviour_mut().gossipsub.subscribe(&topic);
        }
        p2p::NetCommand::Unsubscribe { topic } => {
            let topic = gossipsub::IdentTopic::new(topic.as_str());
            let _ = swarm.behaviour_mut().gossipsub.unsubscribe(&topic);
        }
        p2p::NetCommand::SubscribeRoom { room } => {
            p2p::swarm_loop::subscribe_room(swarm, event_tx, subscriptions, room);
        }
        p2p::NetCommand::UnsubscribeRoom { room } => {
            p2p::swarm_loop::unsubscribe_room(swarm, subscriptions, &room);
        }
        p2p::NetCommand::FetchSnapshot {
            peer_id,
            room,
            part,
        } => {
            if let Ok(libp2p_id) = peer_id.parse::<libp2p::PeerId>() {
                swarm
                    .behaviour_mut()
                    .room_sync
                    .send_request(&libp2p_id, p2p::SnapshotRequest::Fetch { room, part });
            }
        }
        p2p::NetCommand::FetchReplicaSlice {
            peer_id,
            room,
            index,
        } => {
            if let Ok(libp2p_id) = peer_id.parse::<libp2p::PeerId>() {
                swarm.behaviour_mut().room_sync.send_request(
                    &libp2p_id,
                    p2p::SnapshotRequest::FetchReplicaSlice { room, index },
                );
            }
        }
        p2p::NetCommand::RespondSnapshot { id, frame } => {
            if let Some(channel) = snapshot_replies.remove(id) {
                let _ = swarm
                    .behaviour_mut()
                    .room_sync
                    .send_response(channel, frame);
            }
        }
        p2p::NetCommand::Publish { topic, data } => {
            let topic = gossipsub::IdentTopic::new(topic.as_str());
            if let Err(error) = swarm.behaviour_mut().gossipsub.publish(topic, data) {
                event_tx
                    .send(p2p::Event::Error(format!("gossip publish: {error}")))
                    .ok();
            }
        }
        p2p::NetCommand::PeerExchange { peer_id, data } => {
            if let Ok(libp2p_id) = peer_id.parse::<libp2p::PeerId>() {
                swarm
                    .behaviour_mut()
                    .peer_exchange
                    .send_request(&libp2p_id, data);
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
            &identity::GameId::from("test"),
            &p2p::GossipsubConfig::default(),
        )
        .unwrap()
    }

    fn subscriptions() -> p2p::swarm_loop::RoomSubscriptions {
        p2p::swarm_loop::RoomSubscriptions::new(identity::GameId::from("game"))
    }

    fn snapshot_replies() -> p2p::swarm_loop::PendingSnapshotReplies {
        p2p::swarm_loop::PendingSnapshotReplies::default()
    }

    #[tokio::test]
    async fn test_usage() {
        let mut swarm = swarm();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut subscriptions = subscriptions();
        let topic = identity::Topic::from("lobby/topic");
        dispatch_net_command(
            &mut swarm,
            &tx,
            &mut subscriptions,
            &mut snapshot_replies(),
            p2p::NetCommand::Subscribe {
                topic: topic.clone(),
            },
        );
        let hash = IdentTopic::new(topic.as_str()).hash();
        assert!(swarm.behaviour().gossipsub.topics().any(|t| *t == hash));
    }

    #[tokio::test]
    async fn room_subscribe_and_unsubscribe_track_the_map() {
        let mut swarm = swarm();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut subscriptions = subscriptions();
        let room = identity::RoomName::from("room-a");
        let hash =
            gossipsub::IdentTopic::new(p2p::room_topic(&subscriptions.game, &room).as_str()).hash();
        dispatch_net_command(
            &mut swarm,
            &tx,
            &mut subscriptions,
            &mut snapshot_replies(),
            p2p::NetCommand::SubscribeRoom { room: room.clone() },
        );
        assert_eq!(subscriptions.room(&hash), Some(&room));
        assert!(swarm.behaviour().gossipsub.topics().any(|t| *t == hash));
        dispatch_net_command(
            &mut swarm,
            &tx,
            &mut subscriptions,
            &mut snapshot_replies(),
            p2p::NetCommand::UnsubscribeRoom { room },
        );
        assert!(subscriptions.room(&hash).is_none());
        assert!(!swarm.behaviour().gossipsub.topics().any(|t| *t == hash));
    }

    #[tokio::test]
    async fn raw_unsubscribe_leaves_the_swarm() {
        let mut swarm = swarm();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut subscriptions = subscriptions();
        let topic = identity::Topic::from("lobby/topic");
        let hash = IdentTopic::new(topic.as_str()).hash();
        dispatch_net_command(
            &mut swarm,
            &tx,
            &mut subscriptions,
            &mut snapshot_replies(),
            p2p::NetCommand::Subscribe {
                topic: topic.clone(),
            },
        );
        assert!(swarm.behaviour().gossipsub.topics().any(|t| *t == hash));
        dispatch_net_command(
            &mut swarm,
            &tx,
            &mut subscriptions,
            &mut snapshot_replies(),
            p2p::NetCommand::Unsubscribe { topic },
        );
        assert!(!swarm.behaviour().gossipsub.topics().any(|t| *t == hash));
    }
}
