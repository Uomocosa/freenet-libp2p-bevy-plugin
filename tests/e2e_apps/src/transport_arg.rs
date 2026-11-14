use clap::ValueEnum;
use freenet_libp2p_bevy_plugin::p2p::TransportMode;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TransportArg {
    Tcp,
    Quic,
    Both,
}

impl TransportArg {
    #[must_use]
    pub const fn mode(self) -> TransportMode {
        match self {
            Self::Tcp => TransportMode::Tcp,
            Self::Quic => TransportMode::Quic,
            Self::Both => TransportMode::Both,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TransportArg;
    use freenet_libp2p_bevy_plugin::p2p::TransportMode;

    #[test]
    fn test_usage() {
        assert_eq!(TransportArg::Tcp.mode(), TransportMode::Tcp);
        assert_eq!(TransportArg::Quic.mode(), TransportMode::Quic);
        assert_eq!(TransportArg::Both.mode(), TransportMode::Both);
    }
}
