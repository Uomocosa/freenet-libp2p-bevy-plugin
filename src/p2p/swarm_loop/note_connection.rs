use libp2p::swarm::SwarmEvent;

use crate::net_id;
use crate::p2p;

pub fn note_connection<T: p2p::Message>(
    swarm: &libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    event: SwarmEvent<p2p::behaviour::BehaviourEvent<T>>,
) {
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
                event_tx
                    .send(p2p::Event::Net(p2p::NetEvent::PeerConnected(
                        net_id::PeerId(peer_id.to_string()),
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
                swarm,
                &peer_id,
                connection_id,
                &endpoint,
                num_established,
                cause.as_ref(),
            );
            if num_established == 0 {
                event_tx
                    .send(p2p::Event::Net(p2p::NetEvent::PeerDisconnected(
                        net_id::PeerId(peer_id.to_string()),
                    )))
                    .ok();
            }
        }
        _ => {}
    }
}

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
fn log_closed<T: p2p::Message>(
    swarm: &libp2p::Swarm<p2p::Behaviour<T>>,
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
            still_connected = swarm.is_connected(peer_id),
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
            still_connected = swarm.is_connected(peer_id),
            remaining,
            "p2p connection closed",
        );
    }
}

// no test_usage necessary
