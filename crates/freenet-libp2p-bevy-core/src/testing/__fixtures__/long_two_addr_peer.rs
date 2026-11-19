use crate::identity;
use crate::testing;

#[must_use]
pub fn long_two_addr_peer() -> identity::Peer {
    testing::peer(
        &testing::long_peer_id(),
        &[
            "/ip4/192.168.1.20/tcp/51234",
            "/ip4/192.168.1.20/udp/51235/quic-v1",
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::long_two_addr_peer;

    #[test]
    fn test_usage() {
        assert_eq!(long_two_addr_peer().addrs.len(), 2);
    }
}
