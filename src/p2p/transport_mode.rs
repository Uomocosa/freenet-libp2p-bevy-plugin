use atomic_delegate_macros::atomic_delegates;
use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportMode {
    Tcp,
    Quic,
    Both,
}

#[atomic_delegates]
impl TransportMode {
    pub fn matches_transport(&self, addr: &Multiaddr) -> bool {}
}

#[cfg(test)]
mod tests {
    use super::TransportMode;

    #[test]
    fn test_usage() {
        assert_eq!(TransportMode::Tcp, TransportMode::Tcp);
    }
}
