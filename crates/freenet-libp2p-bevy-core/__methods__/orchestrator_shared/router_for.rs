use crate::identity;
use crate::testing;

pub fn router_for<'a>(
    shared: &'a mut testing::orchestrator_shared::OrchestratorShared,
    game: &identity::GameId,
) -> &'a mut testing::PeerMessageRouter {
    shared
        .routers
        .entry(game.clone())
        .or_insert_with(|| testing::PeerMessageRouter::new(game.clone()))
}

#[cfg(test)]
mod tests {
    use super::router_for;
    use crate::identity;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut shared = testing::orchestrator_shared::OrchestratorShared::default();
        let game = identity::GameId::from("game");
        router_for(&mut shared, &game);
        assert!(shared.routers.contains_key(&game));
    }
}
