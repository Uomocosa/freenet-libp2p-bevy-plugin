use atomic_delegate_macros::atomic_delegates;
use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Deref, Display, From,
)]
#[cfg_attr(feature = "bevy", derive(bevy_reflect::Reflect))]
#[cfg_attr(feature = "bevy", reflect(Hash))]
#[from(forward)]
pub struct PeerId(pub String);

#[atomic_delegates]
impl PeerId {
    pub fn peer_author(&self) -> u16 {}
}

#[cfg(test)]
mod tests {
    use super::PeerId;

    #[test]
    fn test_usage() {
        assert_eq!(PeerId::from("peer").as_str(), "peer");
    }
}
