use crate::p2p;
use libp2p::gossipsub;
use libp2p::swarm::dial_opts::PeerCondition;

pub fn dispatch_libp2p_command<Payload: p2p::traits::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    command: p2p::Libp2pCommand,
) {
    match command {
        p2p::Libp2pCommand::Dial(peer) => {
            state.dial_peer(
                &peer.id,
                &peer.addrs,
                PeerCondition::DisconnectedAndNotDialing,
            );
        }
        p2p::Libp2pCommand::ReserveRelay { relay_addr } => {
            if let Ok(addr) = relay_addr.parse::<libp2p::Multiaddr>() {
                let _ = state.swarm.listen_on(addr);
            }
        }
        p2p::Libp2pCommand::SetMdns { mode } => {
            let peer_id = *state.swarm.local_peer_id();
            state.swarm.behaviour_mut().mdns = mode.mdns_behaviour(peer_id);
        }
        p2p::Libp2pCommand::AddKadPeer(peer) => {
            state.seed_kad_peer(&peer.id, &peer.addrs);
        }
        p2p::Libp2pCommand::PutHistoryRecord { id, value, ttl } => {
            state.put_history_record(&id, value, ttl);
        }
        p2p::Libp2pCommand::GetHistoryRecord { id } => {
            let key = id.history_record_key();
            state.swarm.behaviour_mut().kademlia.get_record(key);
        }
        p2p::Libp2pCommand::MeasureHistoryStore => {
            let bytes_by_prefix = state.measure_history_store();
            let measured = p2p::Libp2pEvent::HistoryStoreMeasured { bytes_by_prefix };
            state.event_tx.send(p2p::Event::Libp2p(measured)).ok();
        }
        p2p::Libp2pCommand::RemoveHistoryRecords { prefix } => {
            state.remove_history_records(&prefix);
        }
        p2p::Libp2pCommand::Subscribe { topic } => {
            let topic = gossipsub::IdentTopic::new(topic.as_str());
            let _ = state.swarm.behaviour_mut().gossipsub.subscribe(&topic);
        }
        p2p::Libp2pCommand::Unsubscribe { topic } => {
            let topic = gossipsub::IdentTopic::new(topic.as_str());
            let _ = state.swarm.behaviour_mut().gossipsub.unsubscribe(&topic);
        }
        p2p::Libp2pCommand::SubscribeRoom { room } => {
            state.subscribe_room(room);
        }
        p2p::Libp2pCommand::UnsubscribeRoom { room } => {
            state.unsubscribe_room(&room);
        }
        p2p::Libp2pCommand::FetchSnapshot {
            peer_id,
            room,
            part,
        } => {
            state.send_room_sync_request(&peer_id, p2p::SnapshotRequest::Fetch { room, part });
        }
        p2p::Libp2pCommand::FetchReplicaSlice {
            peer_id,
            room,
            index,
        } => {
            state.send_room_sync_request(
                &peer_id,
                p2p::SnapshotRequest::FetchReplicaSlice { room, index },
            );
        }
        p2p::Libp2pCommand::RespondSnapshot { id, frame } => {
            tracing::debug!(target: "p2p", id = ?id, "snapshot respond requested");
            if let Some(channel) = state.snapshot_replies.remove(id) {
                let _ = state
                    .swarm
                    .behaviour_mut()
                    .room_sync
                    .send_response(channel, frame);
            }
        }
        p2p::Libp2pCommand::Publish { topic, data } => {
            state.publish_gossip(&topic, data);
        }
        p2p::Libp2pCommand::PeerExchange { peer_id, data } => {
            if let Ok(libp2p_id) = peer_id.parse::<libp2p::PeerId>() {
                state
                    .swarm
                    .behaviour_mut()
                    .peer_exchange
                    .send_request(&libp2p_id, data);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use libp2p::gossipsub::IdentTopic;

    use super::dispatch_libp2p_command;
    use crate::identity;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, _rx) = p2p::SwarmLoopState::<u32>::for_test();
        let topic = identity::Topic::from("lobby/topic");
        dispatch_libp2p_command(
            &mut state,
            p2p::Libp2pCommand::Subscribe {
                topic: topic.clone(),
            },
        );
        let hash = IdentTopic::new(topic.as_str()).hash();
        assert!(
            state
                .swarm
                .behaviour()
                .gossipsub
                .topics()
                .any(|t| *t == hash)
        );
    }
}
