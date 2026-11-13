use libp2p::multiaddr::Protocol;

use crate::p2p;

pub fn from_multiaddr(address: &libp2p::Multiaddr) -> p2p::ConnectionKind {
    if address
        .iter()
        .any(|protocol| matches!(protocol, Protocol::P2pCircuit))
    {
        return p2p::ConnectionKind::Relayed;
    }
    if address
        .iter()
        .any(|protocol| matches!(protocol, Protocol::QuicV1 | Protocol::Quic))
    {
        return p2p::ConnectionKind::Quic;
    }
    p2p::ConnectionKind::Tcp
}

#[cfg(test)]
mod tests {
    use super::from_multiaddr;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let tcp: libp2p::Multiaddr = "/ip4/10.0.0.2/tcp/4001".parse().expect("tcp");
        assert_eq!(from_multiaddr(&tcp), p2p::ConnectionKind::Tcp);
    }
}
