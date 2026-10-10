use std::time::Duration;

use libp2p::identify;
use libp2p::kad;
use libp2p::mdns;
use libp2p::relay;
use libp2p::request_response;
use libp2p::swarm::SwarmEvent;

use crate::net_id;
use crate::p2p;

pub fn handle_swarm_event<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    room_queries: &std::collections::HashMap<libp2p::kad::QueryId, net_id::RoomName>,
    listen_addrs: &mut Vec<net_id::PeerAddr>,
    ready_deadline: &mut Option<tokio::time::Instant>,
    mode: p2p::TransportMode,
    event: SwarmEvent<p2p::behaviour::BehaviourEvent<T>>,
) {
    match event {
        SwarmEvent::NewListenAddr { address, .. } => {
            listen_addrs.push(net_id::PeerAddr(address.to_string()));
            if ready_deadline.is_none() {
                let now = tokio::time::Instant::now();
                *ready_deadline = now.checked_add(Duration::from_millis(250));
            }
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::RequestResponse(
            request_response::Event::Message { peer, message, .. },
        )) => {
            p2p::swarm_loop::handle_request_response(swarm, event_tx, &peer, message);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Exchange(
            request_response::Event::Message { peer, message, .. },
        )) => {
            p2p::swarm_loop::handle_exchange(swarm, event_tx, &peer, message);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Kademlia(
            kad::Event::OutboundQueryProgressed {
                result: kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(peer_record))),
                ..
            },
        )) => {
            p2p::swarm_loop::handle_found_record(event_tx, peer_record);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Kademlia(
            kad::Event::OutboundQueryProgressed {
                id,
                result:
                    kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders {
                        providers,
                        ..
                    })),
                ..
            },
        )) => {
            p2p::swarm_loop::handle_found_providers(event_tx, room_queries, id, &providers);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Gossipsub(gossip_event)) => {
            p2p::swarm_loop::handle_gossipsub(swarm, event_tx, gossip_event);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::RelayClient(
            relay::client::Event::ReservationReqAccepted { relay_peer_id, .. },
        )) => {
            event_tx
                .send(p2p::Event::Net(p2p::NetEvent::RelayReserved {
                    relay_peer_id: net_id::PeerId(relay_peer_id.to_string()),
                }))
                .ok();
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Identify(
            identify::Event::Received { info, .. },
        )) => {
            event_tx
                .send(p2p::Event::Net(p2p::NetEvent::ObservedAddr(
                    net_id::PeerAddr(info.observed_addr.to_string()),
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
