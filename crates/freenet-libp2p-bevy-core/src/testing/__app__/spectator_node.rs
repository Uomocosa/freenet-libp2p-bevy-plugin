use crate::identity;
use crate::p2p;
use crate::plugin;
use crate::replica_sync;
use crate::spectator;
use crate::testing;

#[must_use]
pub fn spectator_node(
    game: &identity::GameId,
    history: Option<replica_sync::HistoryConfig>,
) -> testing::TestApp {
    let mut app = testing::TestApp::new();
    app.add_plugins(plugin::P2PPlugin(
        plugin::Config::<u8>::new(p2p::TransportMode::Both, p2p::MdnsMode::Disabled)
            .with_game(game.clone()),
    ));
    app.add_plugins(
        replica_sync::SyncPlugin::<testing::KeyValueMap>::new(game.clone())
            .with_anti_entropy_config(None)
            .with_history_config(history),
    );
    app.add_plugins(spectator::SpectatorPlugin::<testing::KeyValueMap>::new(
        game.clone(),
    ));
    app.update();
    app
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::spectator;
    use crate::testing;

    use super::spectator_node;

    #[tokio::test]
    async fn test_usage() {
        let mut node = spectator_node(&identity::GameId::from("game"), None);
        node.tick(1);
        assert!(
            node.world()
                .get_resource::<spectator::Machine<testing::KeyValueOp>>()
                .is_some()
        );
    }
}
