use crate::identity;
use crate::testing;

pub fn is_connected(
    router: &testing::PeerMessageRouter,
    first: &identity::PeerId,
    second: &identity::PeerId,
) -> bool {
    router
        .connected_peers
        .get(first)
        .is_some_and(|peers| peers.contains(second))
}

#[cfg(test)]
mod tests {
    use super::is_connected;
    use crate::identity;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let first = identity::PeerId::from("a");
        let second = identity::PeerId::from("b");
        router
            .connected_peers
            .entry(first.clone())
            .or_default()
            .insert(second.clone());
        assert!(is_connected(&router, &first, &second));
        assert!(!is_connected(&router, &second, &first));
    }
}
