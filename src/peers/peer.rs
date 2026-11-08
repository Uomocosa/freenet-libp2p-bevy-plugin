use bevy::prelude::Component;
use derive_more::{Deref, From};

use crate::identity;

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref, From)]
pub struct Peer(pub identity::PeerId);

#[rustfmt::skip]
impl PartialEq<identity::PeerId> for Peer {
    fn eq(&self, other: &identity::PeerId) -> bool {
        **self == *other
    }
}

#[cfg(test)]
mod tests {
    use super::Peer;
    use crate::identity;

    #[test]
    fn test_usage() {
        let peer = Peer::from(identity::PeerId::from("alice"));
        assert_eq!(peer, identity::PeerId::from("alice"));
        assert_ne!(peer, identity::PeerId::from("bob"));
    }
}
