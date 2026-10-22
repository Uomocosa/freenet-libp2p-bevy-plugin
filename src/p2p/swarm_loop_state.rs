use std::collections::BTreeMap;
use std::time::Duration;

use libp2p::gossipsub;
use libp2p::identify;
use libp2p::kad;
use libp2p::mdns;
use libp2p::relay;
use libp2p::request_response;
use libp2p::swarm::SwarmEvent;
use libp2p::swarm::dial_opts::{DialOpts, PeerCondition};

use atomic_delegate_macros::atomic_delegates;

use crate::identity;
use crate::p2p;

pub struct SwarmLoopState<Payload: p2p::Message> {
    pub(crate) swarm: libp2p::Swarm<p2p::Behaviour<Payload>>,
    pub(crate) event_tx: tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
    pub(crate) subscriptions: p2p::swarm_loop::RoomSubscriptions,
    pub(crate) snapshot_replies: p2p::swarm_loop::PendingSnapshotReplies,
    pub(crate) listen_addrs: Vec<identity::PeerAddr>,
    pub(crate) ready_deadline: Option<tokio::time::Instant>,
    pub(crate) mode: p2p::TransportMode,
}

impl<Payload: p2p::Message> SwarmLoopState<Payload> {
    #[must_use]
    pub const fn new(
        swarm: libp2p::Swarm<p2p::Behaviour<Payload>>,
        event_tx: tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
        subscriptions: p2p::swarm_loop::RoomSubscriptions,
        snapshot_replies: p2p::swarm_loop::PendingSnapshotReplies,
        mode: p2p::TransportMode,
    ) -> Self {
        Self {
            swarm,
            event_tx,
            subscriptions,
            snapshot_replies,
            listen_addrs: Vec::new(),
            ready_deadline: None,
            mode,
        }
    }
}

#[atomic_delegates]
impl<Payload: p2p::Message> SwarmLoopState<Payload> {
    pub fn dial_peer(
        &mut self,
        peer_id: &identity::PeerId,
        addrs: &[identity::PeerAddr],
        condition: PeerCondition,
    ) {
    }
    pub fn dispatch_command(
        &mut self,
        config: &p2p::GossipsubConfig,
        cmd: Option<p2p::Command<Payload>>,
    ) -> bool {
    }
    pub fn handle_kademlia_query_result(&mut self, result: kad::QueryResult) {}
    pub fn log_mesh_snapshot(&self) {}
    pub fn measure_history_store(
        &mut self,
    ) -> BTreeMap<p2p::HistoryKeyPrefix, p2p::HistoryByteCount> {
    }
    pub fn put_history_record(
        &mut self,
        id: &p2p::RoomHistoryRecordKind,
        value: Vec<u8>,
        ttl: Duration,
    ) {
    }
    pub fn remove_history_records(&mut self, prefix: &p2p::HistoryKeyPrefix) {}
    pub fn seed_kad_peer(&mut self, peer_id: &identity::PeerId, addrs: &[identity::PeerAddr]) {}
    pub fn subscribe_room(&mut self, room: identity::RoomName) {}
    pub fn unsubscribe_room(&mut self, room: &identity::RoomName) {}
}

#[rustfmt::skip]
impl<Payload: p2p::Message> SwarmLoopState<Payload> {
pub fn dial_mdns_peers(&mut self, peers: Vec<(libp2p::PeerId, libp2p::Multiaddr)>) {
    for (peer, addr) in peers {
        if !self.mode.matches_transport(&addr) {
            continue;
        }
        let opts = DialOpts::peer_id(peer)
            .condition(PeerCondition::DisconnectedAndNotDialing)
            .addresses(vec![addr])
            .build();
        if let Err(e) = self.swarm.dial(opts) {
            tracing::debug!(target: "p2p", error = %e, "mdns dial skipped");
        }
    }
}
pub fn dispatch_net_command(&mut self, command: p2p::NetCommand) {
    match command {
        p2p::NetCommand::Dial(peer) => {
            self.dial_peer(
                &peer.id,
                &peer.addrs,
                PeerCondition::DisconnectedAndNotDialing,
            );
        }
        p2p::NetCommand::ReserveRelay { relay_addr } => {
            if let Ok(addr) = relay_addr.parse::<libp2p::Multiaddr>() {
                let _ = self.swarm.listen_on(addr);
            }
        }
        p2p::NetCommand::SetMdns { mode } => {
            let peer_id = *self.swarm.local_peer_id();
            self.swarm.behaviour_mut().mdns = mode.mdns_behaviour(peer_id);
        }
        p2p::NetCommand::AddKadPeer(peer) => {
            self.seed_kad_peer( &peer.id, &peer.addrs);
        }
        p2p::NetCommand::PutHistoryRecord { id, value, ttl } => {
            self.put_history_record( &id, value, ttl);
        }
        p2p::NetCommand::GetHistoryRecord { id } => {
            let key = id.history_record_key();
            self.swarm.behaviour_mut().kademlia.get_record(key);
        }
        p2p::NetCommand::MeasureHistoryStore => {
            let bytes_by_prefix = self.measure_history_store();
            let measured = p2p::NetEvent::HistoryStoreMeasured { bytes_by_prefix };
            self.event_tx.send(p2p::Event::Net(measured)).ok();
        }
        p2p::NetCommand::RemoveHistoryRecords { prefix } => {
            self.remove_history_records( &prefix);
        }
        p2p::NetCommand::Subscribe { topic } => {
            let topic = gossipsub::IdentTopic::new(topic.as_str());
            let _ = self.swarm.behaviour_mut().gossipsub.subscribe(&topic);
        }
        p2p::NetCommand::Unsubscribe { topic } => {
            let topic = gossipsub::IdentTopic::new(topic.as_str());
            let _ = self.swarm.behaviour_mut().gossipsub.unsubscribe(&topic);
        }
        p2p::NetCommand::SubscribeRoom { room } => {
            self.subscribe_room( room);
        }
        p2p::NetCommand::UnsubscribeRoom { room } => {
            self.unsubscribe_room( &room);
        }
        p2p::NetCommand::FetchSnapshot {
            peer_id,
            room,
            part,
        } => {
            if let Ok(libp2p_id) = peer_id.parse::<libp2p::PeerId>() {
                self.swarm
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
                self.swarm.behaviour_mut().room_sync.send_request(
                    &libp2p_id,
                    p2p::SnapshotRequest::FetchReplicaSlice { room, index },
                );
            }
        }
        p2p::NetCommand::RespondSnapshot { id, frame } => {
            if let Some(channel) = self.snapshot_replies.remove(id) {
                let _ = self.swarm
                    .behaviour_mut()
                    .room_sync
                    .send_response(channel, frame);
            }
        }
        p2p::NetCommand::Publish { topic, data } => {
            let topic = gossipsub::IdentTopic::new(topic.as_str());
            if let Err(error) = self.swarm.behaviour_mut().gossipsub.publish(topic, data) {
                self.event_tx
                    .send(p2p::Event::Error(format!("gossip publish: {error}")))
                    .ok();
            }
        }
        p2p::NetCommand::PeerExchange { peer_id, data } => {
            if let Ok(libp2p_id) = peer_id.parse::<libp2p::PeerId>() {
                self.swarm
                    .behaviour_mut()
                    .peer_exchange
                    .send_request(&libp2p_id, data);
            }
        }
    }
}
pub fn handle_gossipsub(&mut self, gossip_event: gossipsub::Event) {
    match gossip_event {
        gossipsub::Event::Message {
            propagation_source,
            message_id,
            message,
        } => {
            log_gossip_in(self, &propagation_source, &message_id, &message);
            forward_gossip(self, propagation_source, message);
        }
        gossipsub::Event::Subscribed { peer_id, topic } => {
            tracing::debug!(target: "p2p", peer = %peer_id, topic = %topic, "p2p gossip subscribed");
        }
        gossipsub::Event::Unsubscribed { peer_id, topic } => {
            tracing::debug!(target: "p2p", peer = %peer_id, topic = %topic, "p2p gossip unsubscribed");
        }
        gossipsub::Event::SlowPeer { peer_id, .. } => {
            tracing::warn!(target: "p2p", peer = %peer_id, "p2p gossip slow peer");
        }
        gossipsub::Event::GossipsubNotSupported { peer_id } => {
            tracing::warn!(target: "p2p", peer = %peer_id, "p2p gossip not supported");
        }
    }
}
pub fn handle_message(&mut self, peer: &libp2p::PeerId, message: request_response::Message<Payload, Payload>) {
    match message {
        request_response::Message::Request {
            request, channel, ..
        } => {
            let payload_clone = request.clone();
            self.event_tx
                .send(p2p::Event::Message {
                    from: identity::PeerId(peer.to_string()),
                    payload: request,
                })
                .ok();
            let _ = self.swarm
                .behaviour_mut()
                .request_response
                .send_response(channel, payload_clone);
        }
        request_response::Message::Response { response, .. } => {
            self.event_tx
                .send(p2p::Event::Message {
                    from: identity::PeerId(peer.to_string()),
                    payload: response,
                })
                .ok();
        }
    }
}
pub fn handle_peer_exchange(&mut self, peer: &libp2p::PeerId, message: request_response::Message<Vec<u8>, Vec<u8>>) {
    if let request_response::Message::Request {
        request, channel, ..
    } = message
    {
        self.event_tx
            .send(p2p::Event::Net(p2p::NetEvent::PeerExchange {
                from: identity::PeerId(peer.to_string()),
                data: request,
            }))
            .ok();
        let _ = self.swarm
            .behaviour_mut()
            .peer_exchange
            .send_response(channel, Vec::new());
    }
}
pub fn handle_room_sync(&mut self, peer: &libp2p::PeerId, message: request_response::Message<p2p::SnapshotRequest, p2p::SnapshotResponse>) {
    match message {
        request_response::Message::Request {
            request, channel, ..
        } => {
            let id = self.snapshot_replies.insert(channel);
            let from = identity::PeerId(peer.to_string());
            match request {
                p2p::SnapshotRequest::Fetch { room, part } => {
                    self.event_tx
                        .send(p2p::Event::Net(p2p::NetEvent::SnapshotRequest {
                            id,
                            from,
                            room,
                            part,
                        }))
                        .ok();
                }
                p2p::SnapshotRequest::FetchReplicaSlice { room, index } => {
                    self.event_tx
                        .send(p2p::Event::Net(p2p::NetEvent::ReplicaSliceRequest {
                            id,
                            from,
                            room,
                            index,
                        }))
                        .ok();
                }
            }
        }
        request_response::Message::Response { response, .. } => {
            let from = identity::PeerId(peer.to_string());
            let event = match response {
                p2p::SnapshotResponse::Part {
                    room,
                    part,
                    total,
                    data,
                } => p2p::NetEvent::SnapshotPart {
                    from,
                    room,
                    part,
                    total,
                    data,
                },
                p2p::SnapshotResponse::Missing { room } => {
                    p2p::NetEvent::SnapshotMissing { from, room }
                }
                p2p::SnapshotResponse::ReplicaSlice { room, index, data } => {
                    p2p::NetEvent::ReplicaSlice {
                        from,
                        room,
                        index,
                        data,
                    }
                }
            };
            self.event_tx.send(p2p::Event::Net(event)).ok();
        }
    }
}
pub fn handle_swarm_event(&mut self, event: SwarmEvent<p2p::behaviour::BehaviourEvent<Payload>>) {
    match event {
        SwarmEvent::NewListenAddr { address, .. } => {
            self.listen_addrs.push(identity::PeerAddr(address.to_string()));
            if self.ready_deadline.is_none() {
                let now = tokio::time::Instant::now();
                self.ready_deadline = now.checked_add(Duration::from_millis(250));
            }
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::RequestResponse(
            request_response::Event::Message { peer, message, .. },
        )) => {
            self.handle_message( &peer, message);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::PeerExchange(
            request_response::Event::Message { peer, message, .. },
        )) => {
            self.handle_peer_exchange( &peer, message);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::RoomSync(
            request_response::Event::Message { peer, message, .. },
        )) => {
            self.handle_room_sync( &peer, message);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::RoomSync(
            request_response::Event::OutboundFailure { peer, .. },
        )) => {
            self.event_tx
                .send(p2p::Event::Net(p2p::NetEvent::SnapshotFailed {
                    from: identity::PeerId(peer.to_string()),
                }))
                .ok();
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Kademlia(
            kad::Event::OutboundQueryProgressed { result, .. },
        )) => {
            self.handle_kademlia_query_result( result);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Gossipsub(gossip_event)) => {
            self.handle_gossipsub( gossip_event);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::RelayClient(
            relay::client::Event::ReservationReqAccepted { relay_peer_id, .. },
        )) => {
            self.event_tx
                .send(p2p::Event::Net(p2p::NetEvent::RelayReserved {
                    relay_peer_id: identity::PeerId(relay_peer_id.to_string()),
                }))
                .ok();
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Identify(
            identify::Event::Received { info, .. },
        )) => {
            self.event_tx
                .send(p2p::Event::Net(p2p::NetEvent::ObservedAddr(
                    identity::PeerAddr(info.observed_addr.to_string()),
                )))
                .ok();
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Mdns(mdns::Event::Discovered(
            peers,
        ))) => {
            self.dial_mdns_peers(peers);
        }
        event
        @ (SwarmEvent::ConnectionEstablished { .. } | SwarmEvent::ConnectionClosed { .. }) => {
            self.note_connection( event);
        }
        SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
            self.note_dial_failure( peer_id, &error);
        }
        _ => {}
    }
}
pub fn note_connection(&mut self, event: SwarmEvent<p2p::behaviour::BehaviourEvent<Payload>>) {
    match event {
        SwarmEvent::ConnectionEstablished {
            peer_id,
            connection_id,
            endpoint,
            num_established,
            ..
        } => {
            log_established(&peer_id, connection_id, &endpoint);
            if num_established.get() == 1 {
                self.event_tx
                    .send(p2p::Event::Net(p2p::NetEvent::PeerConnected(
                        identity::PeerId(peer_id.to_string()),
                    )))
                    .ok();
            }
        }
        SwarmEvent::ConnectionClosed {
            peer_id,
            connection_id,
            endpoint,
            num_established,
            cause,
            ..
        } => {
            log_closed(
                self,
                &peer_id,
                connection_id,
                &endpoint,
                num_established,
                cause.as_ref(),
            );
            if num_established == 0 {
                self.event_tx
                    .send(p2p::Event::Net(p2p::NetEvent::PeerDisconnected(
                        identity::PeerId(peer_id.to_string()),
                    )))
                    .ok();
            }
        }
        _ => {}
    }
}
pub fn note_dial_failure(&mut self, peer_id: Option<libp2p::PeerId>, error: &libp2p::swarm::DialError) {
    match peer_id {
        Some(peer_id) => {
            self.event_tx
                .send(p2p::Event::Net(p2p::NetEvent::DialFailed {
                    peer_id: identity::PeerId(peer_id.to_string()),
                    reason: error.to_string(),
                }))
                .ok();
        }
        None => {
            tracing::debug!(target: "p2p", error = %error, "p2p dial failed without peer");
        }
    }
}
}

// no test_usage necessary

// needed helper: samples gossip mesh depth for one inbound message
fn log_gossip_in<Payload: p2p::Message>(
    state: &p2p::SwarmLoopState<Payload>,
    propagation_source: &libp2p::PeerId,
    message_id: &libp2p::gossipsub::MessageId,
    message: &gossipsub::Message,
) {
    let mesh = state
        .swarm
        .behaviour()
        .gossipsub
        .mesh_peers(&message.topic)
        .count();
    tracing::debug!(
        target: "p2p",
        source = %propagation_source,
        message = %message_id,
        topic = %message.topic,
        mesh,
        "p2p gossip received"
    );
}

// needed helper: converts a gossipsub wire message into a Bevy event
fn forward_gossip<Payload: p2p::Message>(
    state: &p2p::SwarmLoopState<Payload>,
    propagation_source: libp2p::PeerId,
    message: gossipsub::Message,
) {
    let from = identity::PeerId(message.source.unwrap_or(propagation_source).to_string());
    let Some(room) = state.subscriptions.room(&message.topic) else {
        state
            .event_tx
            .send(p2p::Event::Net(p2p::NetEvent::Gossip {
                topic: identity::Topic(message.topic.to_string()),
                from,
                data: message.data,
            }))
            .ok();
        return;
    };
    match bincode::deserialize::<Payload>(&message.data) {
        Ok(payload) => {
            state
                .event_tx
                .send(p2p::Event::RoomMessage {
                    room: room.clone(),
                    from,
                    payload,
                })
                .ok();
        }
        Err(error) => {
            state
                .event_tx
                .send(p2p::Event::Error(format!(
                    "room payload decode failed: {error}"
                )))
                .ok();
        }
    }
}

// no test_usage necessary

// no test_usage necessary

// no test_usage necessary

// no test_usage necessary

// needed helper: records one established connection with its endpoint
fn log_established(
    peer_id: &libp2p::PeerId,
    connection_id: libp2p::swarm::ConnectionId,
    endpoint: &libp2p::core::ConnectedPoint,
) {
    tracing::info!(
        target: "p2p",
        peer = %peer_id,
        connection = %connection_id,
        address = %endpoint.get_remote_address(),
        dialer = endpoint.is_dialer(),
        "p2p connection established"
    );
}

// needed helper: records one closed connection and whether the peer stays up
fn log_closed<Payload: p2p::Message>(
    state: &p2p::SwarmLoopState<Payload>,
    peer_id: &libp2p::PeerId,
    connection_id: libp2p::swarm::ConnectionId,
    endpoint: &libp2p::core::ConnectedPoint,
    remaining: u32,
    cause: Option<&libp2p::swarm::ConnectionError>,
) {
    if let Some(reason) = cause {
        tracing::info!(
            target: "p2p",
            peer = %peer_id,
            connection = %connection_id,
            address = %endpoint.get_remote_address(),
            dialer = endpoint.is_dialer(),
            still_connected = state.swarm.is_connected(peer_id),
            remaining,
            reason = %reason,
            "p2p connection closed",
        );
    } else {
        tracing::info!(
            target: "p2p",
            peer = %peer_id,
            connection = %connection_id,
            address = %endpoint.get_remote_address(),
            dialer = endpoint.is_dialer(),
            still_connected = state.swarm.is_connected(peer_id),
            remaining,
            "p2p connection closed",
        );
    }
}

// no test_usage necessary

#[cfg(test)]
mod tests {

    use super::SwarmLoopState;
    use crate::identity;
    use crate::p2p;
    use libp2p::gossipsub;
    use libp2p::gossipsub::IdentTopic;
    use libp2p::identity::Keypair;
    use libp2p::swarm::DialError;

    impl<Payload: p2p::Message> SwarmLoopState<Payload> {
        pub(crate) fn for_test() -> (
            Self,
            tokio::sync::mpsc::UnboundedReceiver<p2p::Event<Payload>>,
        ) {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            let config = p2p::RunnerConfig {
                mode: p2p::TransportMode::Both,
                mdns: p2p::MdnsMode::Disabled,
                game: identity::GameId::from("test"),
                gossipsub: p2p::GossipsubConfig::default(),
            };
            let swarm =
                p2p::RunnerConfig::build_swarm::<Payload>(&config, Keypair::generate_ed25519())
                    .unwrap();
            let state = Self::new(
                swarm,
                tx,
                p2p::swarm_loop::RoomSubscriptions::new(config.game.clone()),
                p2p::swarm_loop::PendingSnapshotReplies::default(),
                config.mode,
            );
            (state, rx)
        }
    }

    #[tokio::test]
    async fn test_usage() {
        let (mut state, _rx) = p2p::SwarmLoopState::<u32>::for_test();
        let topic = identity::Topic::from("lobby/topic");
        state.dispatch_net_command(p2p::NetCommand::Subscribe {
            topic: topic.clone(),
        });
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

    #[tokio::test]
    async fn room_subscribe_and_unsubscribe_track_the_map() {
        let (mut state, _rx) = p2p::SwarmLoopState::<u32>::for_test();
        let room = identity::RoomName::from("room-a");
        let hash = gossipsub::IdentTopic::new(
            identity::Topic::from_room(&state.subscriptions.game, &room).as_str(),
        )
        .hash();
        state.dispatch_net_command(p2p::NetCommand::SubscribeRoom { room: room.clone() });
        assert_eq!(state.subscriptions.room(&hash), Some(&room));
        assert!(
            state
                .swarm
                .behaviour()
                .gossipsub
                .topics()
                .any(|t| *t == hash)
        );
        state.dispatch_net_command(p2p::NetCommand::UnsubscribeRoom { room });
        assert!(state.subscriptions.room(&hash).is_none());
        assert!(
            !state
                .swarm
                .behaviour()
                .gossipsub
                .topics()
                .any(|t| *t == hash)
        );
    }

    #[tokio::test]
    async fn raw_unsubscribe_leaves_the_swarm() {
        let (mut state, _rx) = p2p::SwarmLoopState::<u32>::for_test();
        let topic = identity::Topic::from("lobby/topic");
        let hash = IdentTopic::new(topic.as_str()).hash();
        state.dispatch_net_command(p2p::NetCommand::Subscribe {
            topic: topic.clone(),
        });
        assert!(
            state
                .swarm
                .behaviour()
                .gossipsub
                .topics()
                .any(|t| *t == hash)
        );
        state.dispatch_net_command(p2p::NetCommand::Unsubscribe { topic });
        assert!(
            !state
                .swarm
                .behaviour()
                .gossipsub
                .topics()
                .any(|t| *t == hash)
        );
    }

    fn message_event(
        propagation_source: &libp2p::PeerId,
        source: Option<libp2p::PeerId>,
        data: Vec<u8>,
        topic: gossipsub::TopicHash,
    ) -> gossipsub::Event {
        gossipsub::Event::Message {
            propagation_source: *propagation_source,
            message_id: gossipsub::MessageId::new(b"m1"),
            message: gossipsub::Message {
                source,
                data,
                sequence_number: None,
                topic,
            },
        }
    }

    #[test]
    fn test_usage_1() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        let relay = libp2p::PeerId::random();
        let event = message_event(
            &relay,
            None,
            vec![7],
            gossipsub::TopicHash::from_raw("lobby/topic"),
        );
        state.handle_gossipsub(event);
        let Ok(p2p::Event::Net(p2p::NetEvent::Gossip { topic, from, data })) = rx.try_recv() else {
            panic!("expected a gossip event");
        };
        assert_eq!((topic.as_str(), data), ("lobby/topic", vec![7]));
        assert_eq!(from.as_str(), relay.to_string());
    }

    #[test]
    fn registered_room_topic_becomes_a_room_message() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        let room = identity::RoomName::from("room-a");
        let topic = gossipsub::IdentTopic::new(
            identity::Topic::from_room(&state.subscriptions.game, &room).as_str(),
        );
        state.subscriptions.register(&topic, room.clone());
        let data = bincode::serialize(&9u32).unwrap();
        state.handle_gossipsub(message_event(
            &libp2p::PeerId::random(),
            Some(libp2p::PeerId::random()),
            data,
            topic.hash(),
        ));
        let Ok(p2p::Event::RoomMessage {
            room: got, payload, ..
        }) = rx.try_recv()
        else {
            panic!("expected a room message");
        };
        assert_eq!((got, payload), (room, 9));
    }

    #[test]
    fn undecodable_room_payload_is_an_error() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        let room = identity::RoomName::from("room-a");
        let topic = gossipsub::IdentTopic::new(
            identity::Topic::from_room(&state.subscriptions.game, &room).as_str(),
        );
        state.subscriptions.register(&topic, room);
        state.handle_gossipsub(message_event(
            &libp2p::PeerId::random(),
            None,
            Vec::new(),
            topic.hash(),
        ));
        let Ok(p2p::Event::Error(message)) = rx.try_recv() else {
            panic!("expected an error event");
        };
        assert!(message.contains("decode failed"));
    }

    #[test]
    fn test_usage_2() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        let peer = libp2p::PeerId::random();
        state.note_dial_failure(Some(peer), &DialError::NoAddresses);
        let Ok(p2p::Event::Net(p2p::NetEvent::DialFailed { peer_id, .. })) = rx.try_recv() else {
            panic!("expected a dial failure");
        };
        assert_eq!(peer_id.as_str(), peer.to_string());
        state.note_dial_failure(None, &DialError::NoAddresses);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn test_usage_3() {
        let (state, _rx) = p2p::SwarmLoopState::<u32>::for_test();
        assert_eq!(state.mode, p2p::TransportMode::Both);
    }
}
