pub fn log_established(
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

#[cfg(test)]
mod tests {
    use libp2p::core::{ConnectedPoint, Endpoint};

    use super::log_established;

    #[test]
    fn test_usage() {
        let peer = libp2p::PeerId::random();
        let endpoint = ConnectedPoint::Dialer {
            address: "/ip4/127.0.0.1/tcp/4003".parse().expect("multiaddr"),
            role_override: Endpoint::Dialer,
            port_use: libp2p::core::transport::PortUse::New,
        };
        log_established(
            &peer,
            libp2p::swarm::ConnectionId::new_unchecked(0),
            &endpoint,
        );
        assert!(endpoint.is_dialer());
    }
}
