use crate::PeerAddr;

pub fn is_unspecified(addr: &PeerAddr) -> bool {
    addr.parse::<multiaddr::Multiaddr>().is_ok_and(|multiaddr| {
        multiaddr
            .iter()
            .any(|protocol| matches!(protocol, multiaddr::Protocol::Ip4(ip) if ip.is_unspecified()))
    })
}

#[cfg(test)]
mod tests {
    use super::is_unspecified;
    use crate::PeerAddr;

    #[test]
    fn test_usage() {
        assert!(is_unspecified(&PeerAddr::from("/ip4/0.0.0.0/tcp/9000")));
        assert!(!is_unspecified(&PeerAddr::from("/ip4/1.2.3.4/tcp/9000")));
    }
}
