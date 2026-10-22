use bevy::prelude::Reflect;
use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};

use crate::identity;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Deref,
    Display,
    From,
    Reflect,
)]
#[reflect(Hash)]
#[from(forward)]
pub struct PeerAddr(pub String);

impl PeerAddr {
    #[must_use]
    pub fn dialable(addrs: Vec<Self>) -> Vec<Self> {
        addrs
            .into_iter()
            .filter(|addr| !is_unspecified(addr))
            .collect()
    }
}

// needed helper: a multiaddr offering 0.0.0.0 cannot be dialled
fn is_unspecified(addr: &identity::PeerAddr) -> bool {
    addr.parse::<libp2p::Multiaddr>().is_ok_and(|multiaddr| {
        multiaddr.iter().any(|protocol| {
                matches!(protocol, libp2p::multiaddr::Protocol::Ip4(ip) if ip.is_unspecified())
            })
    })
}

#[cfg(test)]
mod tests {
    use super::PeerAddr;
    use crate::identity;

    #[test]
    fn test_usage() {
        assert_eq!(
            PeerAddr::dialable(vec![identity::PeerAddr::from("/ip4/0.0.0.0/tcp/9000")]),
            Vec::<identity::PeerAddr>::new()
        );
        assert_eq!(
            PeerAddr::dialable(vec![identity::PeerAddr::from("/ip4/1.2.3.4/tcp/9000")]).len(),
            1
        );
    }
}
