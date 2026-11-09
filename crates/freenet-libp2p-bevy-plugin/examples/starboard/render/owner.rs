use bevy::prelude::Component;
use freenet_libp2p_bevy_plugin::identity;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum Owner {
    Me,
    Peer(identity::PeerId),
}

#[rustfmt::skip]
impl Owner {
    #[must_use]
    pub const fn is_me(&self) -> bool { matches!(self, Self::Me) }
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_plugin::identity;

    use super::Owner;

    #[test]
    fn test_usage() {
        assert!(Owner::Me.is_me());
        assert!(!Owner::Peer(identity::PeerId::from("peer-a")).is_me());
    }
}
