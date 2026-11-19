use crate::identity;
use crate::testing;

#[must_use]
pub fn registered_peer(router: &mut testing::PeerMessageRouter, id: &str) -> identity::PeerId {
    let peer = testing::peer(id, &[]);
    router.register_peer(&peer);
    peer.id
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::testing;

    use super::registered_peer;

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        assert_eq!(registered_peer(&mut router, "a").as_str(), "a");
    }
}
