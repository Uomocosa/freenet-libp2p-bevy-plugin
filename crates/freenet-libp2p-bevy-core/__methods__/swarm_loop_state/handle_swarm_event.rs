use crate::identity;
use crate::p2p;
use libp2p::identify;
use libp2p::kad;
use libp2p::mdns;
use libp2p::relay;
use libp2p::request_response;
use libp2p::swarm::SwarmEvent;
use std::time::Duration;

pub fn handle_swarm_event<Payload: p2p::traits::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    event: SwarmEvent<p2p::behaviour::BehaviourEvent<Payload>>,
) {
    match event {
        SwarmEvent::NewListenAddr { address, .. } => {
            state
                .listen_addrs
                .push(identity::PeerAddr(address.to_string()));
            if state.ready_deadline.is_none() {
                let now = tokio::time::Instant::now();
                state.ready_deadline = now.checked_add(Duration::from_millis(250));
            }
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::RequestResponse(
            request_response::Event::Message { peer, message, .. },
        )) => {
            state.handle_message(&peer, message);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::PeerExchange(
            request_response::Event::Message { peer, message, .. },
        )) => {
            state.handle_peer_exchange(&peer, message);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::RoomSync(
            request_response::Event::Message { peer, message, .. },
        )) => {
            state.handle_room_sync(&peer, message);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::RoomSync(
            request_response::Event::OutboundFailure {
                peer, request_id, ..
            },
        )) => {
            let Some(request) = state.room_sync_requests_in_flight.remove(&request_id) else {
                tracing::debug!(target: "p2p", peer = %peer, "room sync failure without a tracked request");
                return;
            };
            state
                .event_tx
                .send(p2p::Event::Libp2p(
                    p2p::Libp2pEvent::RoomSyncRequestFailed {
                        from: identity::PeerId(peer.to_string()),
                        request,
                    },
                ))
                .ok();
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Kademlia(
            kad::Event::OutboundQueryProgressed { result, .. },
        )) => {
            state.handle_kademlia_query_result(result);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Gossipsub(gossip_event)) => {
            state.handle_gossipsub(gossip_event);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::RelayClient(
            relay::client::Event::ReservationReqAccepted { relay_peer_id, .. },
        )) => {
            state
                .event_tx
                .send(p2p::Event::Libp2p(p2p::Libp2pEvent::RelayReserved {
                    relay_peer_id: identity::PeerId(relay_peer_id.to_string()),
                }))
                .ok();
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Identify(
            identify::Event::Received { peer_id, info, .. },
        )) => {
            state.handle_identify(&peer_id, &info);
        }
        SwarmEvent::Behaviour(p2p::behaviour::BehaviourEvent::Mdns(mdns::Event::Discovered(
            peers,
        ))) => {
            state.dial_mdns_peers(peers);
        }
        event
        @ (SwarmEvent::ConnectionEstablished { .. } | SwarmEvent::ConnectionClosed { .. }) => {
            state.note_connection(event);
        }
        SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
            state.note_dial_failure(peer_id, &error);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use libp2p::core::transport::ListenerId;
    use libp2p::swarm::SwarmEvent;

    use super::handle_swarm_event;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, _rx) = p2p::SwarmLoopState::<u32>::for_test();
        handle_swarm_event(
            &mut state,
            SwarmEvent::NewListenAddr {
                listener_id: ListenerId::next(),
                address: "/ip4/127.0.0.1/tcp/4001".parse().expect("multiaddr"),
            },
        );
        assert_eq!(state.listen_addrs.len(), 1);
    }
}
