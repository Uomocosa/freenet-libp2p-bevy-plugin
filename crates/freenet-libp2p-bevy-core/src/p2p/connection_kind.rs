use atomic_delegate_macros::atomic_delegates;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConnectionKind {
    Tcp,
    Quic,
    Relayed,
}

#[rustfmt::skip]
impl From<&libp2p::Multiaddr> for ConnectionKind {
    fn from(address: &libp2p::Multiaddr) -> Self {
        use libp2p::multiaddr::Protocol;
        if address.iter().any(|protocol| matches!(protocol, Protocol::P2pCircuit)) {
            return Self::Relayed;
        }
        if address.iter().any(|protocol| matches!(protocol, Protocol::QuicV1 | Protocol::Quic)) {
            return Self::Quic;
        }
        Self::Tcp
    }
}

#[atomic_delegates]
impl ConnectionKind {
    pub fn is_hole_punched(kinds: &[Self]) -> bool {}
    pub fn render_history(kinds: &[Self]) -> String {}
}

#[cfg(test)]
mod tests {
    use super::ConnectionKind;

    #[test]
    fn test_usage() {
        let tcp: libp2p::Multiaddr = "/ip4/10.0.0.2/tcp/4001".parse().expect("tcp");
        assert_eq!(ConnectionKind::from(&tcp), ConnectionKind::Tcp);
        let quic: libp2p::Multiaddr = "/ip4/10.0.0.2/udp/4001/quic-v1".parse().expect("quic");
        assert_eq!(ConnectionKind::from(&quic), ConnectionKind::Quic);
        let relay_peer = libp2p::PeerId::random().to_string();
        let relayed: libp2p::Multiaddr =
            format!("/ip4/10.0.0.2/tcp/4001/p2p/{relay_peer}/p2p-circuit")
                .parse()
                .expect("relayed");
        assert_eq!(ConnectionKind::from(&relayed), ConnectionKind::Relayed);
        assert!(ConnectionKind::Tcp < ConnectionKind::Quic);
        assert!(ConnectionKind::is_hole_punched(&[
            ConnectionKind::Relayed,
            ConnectionKind::Tcp
        ]));
        assert_eq!(
            ConnectionKind::render_history(&[ConnectionKind::Tcp]),
            "Tcp".to_string()
        );
    }
}
