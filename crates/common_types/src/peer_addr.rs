use atomic_delegate_macros::atomic_delegates;
use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Deref, Display, From,
)]
#[cfg_attr(feature = "bevy", derive(bevy_reflect::Reflect))]
#[cfg_attr(feature = "bevy", reflect(Hash))]
#[from(forward)]
pub struct PeerAddr(pub String);

impl PeerAddr {
    #[must_use]
    pub fn dialable(addrs: Vec<Self>) -> Vec<Self> {
        addrs
            .into_iter()
            .filter(|addr| !addr.is_unspecified())
            .collect()
    }
}

#[atomic_delegates]
impl PeerAddr {
    #[must_use]
    pub(crate) fn is_unspecified(&self) -> bool {}
}

#[cfg(test)]
mod tests {
    use super::PeerAddr;

    #[test]
    fn test_usage() {
        assert_eq!(
            PeerAddr::dialable(vec![PeerAddr::from("/ip4/0.0.0.0/tcp/9000")]),
            Vec::<PeerAddr>::new()
        );
        assert_eq!(
            PeerAddr::dialable(vec![PeerAddr::from("/ip4/1.2.3.4/tcp/9000")]).len(),
            1
        );
    }
}
