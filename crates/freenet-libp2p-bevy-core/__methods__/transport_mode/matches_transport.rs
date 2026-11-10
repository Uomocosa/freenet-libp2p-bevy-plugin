use libp2p::Multiaddr;
use libp2p::multiaddr::Protocol;

use crate::p2p;

#[must_use]
pub fn matches_transport(mode: p2p::TransportMode, addr: &Multiaddr) -> bool {
    let is_udp = addr
        .iter()
        .any(|protocol| matches!(protocol, Protocol::Udp(_)));
    match mode {
        p2p::TransportMode::Tcp => !is_udp,
        p2p::TransportMode::Quic => is_udp,
        p2p::TransportMode::Both => true,
    }
}

#[cfg(test)]
mod tests {
    use libp2p::Multiaddr;

    use super::matches_transport;
    use crate::p2p::TransportMode;

    #[test]
    fn test_usage() {
        let tcp: Multiaddr = "/ip4/10.0.0.2/tcp/4001".parse().unwrap();
        let quic: Multiaddr = "/ip4/10.0.0.2/udp/4001/quic-v1".parse().unwrap();
        assert!(matches_transport(TransportMode::Tcp, &tcp));
        assert!(!matches_transport(TransportMode::Tcp, &quic));
        assert!(matches_transport(TransportMode::Quic, &quic));
        assert!(!matches_transport(TransportMode::Quic, &tcp));
        assert!(matches_transport(TransportMode::Both, &tcp));
        assert!(matches_transport(TransportMode::Both, &quic));
    }
}
