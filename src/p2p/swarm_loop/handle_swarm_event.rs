use std::time::Duration;

use libp2p::identify;
use libp2p::kad;
use libp2p::mdns;
use libp2p::relay;
use libp2p::request_response;
use libp2p::swarm::SwarmEvent;

use crate::identity;
use crate::p2p;

pub fn handle_swarm_event<Payload: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<Payload>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
    subscriptions: &p2p::swarm_loop::RoomSubscriptions,
    snapshot_replies: &mut p2p::swarm_loop::PendingSnapshotReplies,
    listen_addrs: &mut Vec<identity::PeerAddr>,
    ready_deadline: &mut Option<tokio::time::Instant>,
    mode: p2p::TransportMode,
    event: SwarmEvent<p2p::behaviour::BehaviourEvent<Payload>>,
) {
    match event {
        SwarmEvent::NewListenAddr { address, .. } => {
            listen_addrs.push(identity::PeerAddr(address.to_string()));
            if ready_deadline.is_none() {
                let now = tokio::time::Instant::now();
                *ready_deadline = now.checked_add(Duration::from_millis(250));
            }
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::RequestResponse(
            request_response::Event::Message { peer, message, .. },
        )) => {
            p2p::swarm_loop::handle_message(swarm, event_tx, &peer, message);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::PeerExchange(
            request_response::Event::Message { peer, message, .. },
        )) => {
            p2p::swarm_loop::handle_peer_exchange(swarm, event_tx, &peer, message);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::RoomSync(
            request_response::Event::Message { peer, message, .. },
        )) => {
            p2p::swarm_loop::handle_room_sync(event_tx, snapshot_replies, &peer, message);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::RoomSync(
            request_response::Event::OutboundFailure { peer, .. },
        )) => {
            event_tx
                .send(p2p::Event::Net(p2p::NetEvent::SnapshotFailed {
                    from: identity::PeerId(peer.to_string()),
                }))
                .ok();
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Kademlia(
            kad::Event::OutboundQueryProgressed { result, .. },
        )) => {
            p2p::swarm_loop::handle_kademlia_query_result(event_tx, result);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Gossipsub(gossip_event)) => {
            p2p::swarm_loop::handle_gossipsub(swarm, event_tx, subscriptions, gossip_event);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::RelayClient(
            relay::client::Event::ReservationReqAccepted { relay_peer_id, .. },
        )) => {
            event_tx
                .send(p2p::Event::Net(p2p::NetEvent::RelayReserved {
                    relay_peer_id: identity::PeerId(relay_peer_id.to_string()),
                }))
                .ok();
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Identify(
            identify::Event::Received { info, .. },
        )) => {
            event_tx
                .send(p2p::Event::Net(p2p::NetEvent::ObservedAddr(
                    identity::PeerAddr(info.observed_addr.to_string()),
                )))
                .ok();
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Mdns(mdns::Event::Discovered(
            peers,
        ))) => {
            p2p::swarm_loop::dial_mdns_peers(swarm, mode, peers);
        }
        event
        @ (SwarmEvent::ConnectionEstablished { .. } | SwarmEvent::ConnectionClosed { .. }) => {
            p2p::swarm_loop::note_connection(swarm, event_tx, event);
        }
        SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
            p2p::swarm_loop::note_dial_failure(event_tx, peer_id, &error);
        }
        _ => {}
    }
}

// no test_usage necessary
