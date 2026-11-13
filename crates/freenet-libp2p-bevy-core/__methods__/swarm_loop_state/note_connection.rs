use crate::identity;
use crate::p2p;
use libp2p::swarm::SwarmEvent;

pub fn note_connection<Payload: p2p::traits::Message>(
    state: &p2p::SwarmLoopState<Payload>,
    event: SwarmEvent<p2p::behaviour::BehaviourEvent<Payload>>,
) {
    match event {
        SwarmEvent::ConnectionEstablished {
            peer_id,
            connection_id,
            endpoint,
            num_established,
            ..
        } => {
            p2p::SwarmLoopState::<Payload>::log_established(&peer_id, connection_id, &endpoint);
            let kind = p2p::ConnectionKind::from(endpoint.get_remote_address());
            state
                .event_tx
                .send(p2p::Event::Libp2p(
                    p2p::Libp2pEvent::Libp2pConnectionEstablished {
                        peer_id: identity::PeerId(peer_id.to_string()),
                        kind,
                    },
                ))
                .ok();
            if num_established.get() == 1 {
                state
                    .event_tx
                    .send(p2p::Event::Libp2p(p2p::Libp2pEvent::PeerConnected(
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
            state.log_closed(
                &peer_id,
                connection_id,
                &endpoint,
                num_established,
                cause.as_ref(),
            );
            if num_established == 0 {
                state
                    .event_tx
                    .send(p2p::Event::Libp2p(p2p::Libp2pEvent::PeerDisconnected(
                        identity::PeerId(peer_id.to_string()),
                    )))
                    .ok();
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use libp2p::core::transport::ListenerId;
    use libp2p::swarm::SwarmEvent;

    use super::note_connection;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (state, _rx) = p2p::SwarmLoopState::<u32>::for_test();
        note_connection(
            &state,
            SwarmEvent::NewListenAddr {
                listener_id: ListenerId::next(),
                address: "/ip4/127.0.0.1/tcp/4002".parse().expect("multiaddr"),
            },
        );
        assert_eq!(state.listen_addrs, Vec::new());
    }
}
