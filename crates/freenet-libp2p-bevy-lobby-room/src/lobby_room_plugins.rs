use bevy::app::{PluginGroup, PluginGroupBuilder};
use derive_more::Deref;

use crate::discovery;
use crate::ui;

#[derive(Deref)]
pub struct LobbyRoomPlugins(pub discovery::Config);

impl PluginGroup for LobbyRoomPlugins {
    fn build(self) -> PluginGroupBuilder {
        let Self(config) = self;
        PluginGroupBuilder::start::<Self>()
            .add(discovery::Plugin::new(config))
            .add(ui::discovery::LobbyRoomUiPlugin)
            .add(ui::direct_dial::FriendsUiPlugin)
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::LobbyRoomPlugins;
    use crate::discovery;
    use crate::ui;

    #[tokio::test]
    async fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(
            LobbyRoomPlugins(discovery::Config::default())
                .build()
                .disable::<ui::discovery::LobbyRoomUiPlugin>()
                .disable::<ui::direct_dial::FriendsUiPlugin>(),
        );
        assert!(app.world().get_resource::<discovery::Config>().is_some());
    }
}
