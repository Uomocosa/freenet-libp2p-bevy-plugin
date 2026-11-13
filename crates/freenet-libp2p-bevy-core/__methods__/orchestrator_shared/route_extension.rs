use crate::identity;
use crate::testing;
use crate::time;

pub fn route_extension(
    shared: &mut testing::orchestrator_shared::OrchestratorShared,
    peer: &identity::PeerId,
    bytes: Vec<u8>,
    now: time::UnixTime,
) -> Vec<testing::FanOut> {
    let Some(game) = shared.peer_games.get(peer).cloned() else {
        return Vec::new();
    };
    shared.kit.on_extension(peer, &game, bytes, now)
}
#[cfg(test)]
mod tests {
    use super::route_extension;
    use crate::identity;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut shared = testing::orchestrator_shared::OrchestratorShared::default();
        let peer = identity::PeerId::from("a");
        let fan_outs = route_extension(&mut shared, &peer, vec![1], time::UnixTime::from_secs(1));
        assert_eq!(fan_outs, Vec::new());
    }
}
