use std::path::PathBuf;

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy::window::Window;
use freenet_libp2p_bevy_plugin::lobby_room::{LobbyRoomPlugins, discovery, peek, ui};
use freenet_libp2p_bevy_plugin::p2p::{MdnsMode, TransportMode};
use freenet_libp2p_bevy_plugin::{direct_dial, identity, plugin, replica_sync};

use crate::starboard::app::StarboardPlugin;
use crate::starboard::canvas::Canvas;
use crate::starboard::room::CursorUpdate;

pub struct StarboardApp {
    pub username: identity::Username,
    pub game: identity::GameId,
    pub transport: TransportMode,
    pub keypair: Option<identity::SavedKeypair>,
    pub peer_book_file: Option<PathBuf>,
    pub freenet_port: u16,
    pub lease: discovery::HolderHeartbeatIntervals,
    pub history: replica_sync::HistoryConfig,
}

impl StarboardApp {
    #[must_use]
    pub fn primary_window(username: &str) -> Window {
        Window {
            title: format!("starboard-{username}"),
            resolution: (960, 540).into(),
            ..default()
        }
    }
}

impl Plugin for StarboardApp {
    fn build(&self, app: &mut App) {
        let base = plugin::Config::<CursorUpdate>::new(self.transport, MdnsMode::Disabled)
            .with_game(self.game.clone());
        let keyed = match &self.keypair {
            Some(saved) => base.with_keypair(saved.clone()),
            None => base,
        };
        app.add_systems(Startup, StarboardPlugin::spawn_camera);
        app.insert_resource(discovery::FreenetPort(self.freenet_port));
        app.add_plugins(plugin::P2PPlugin(keyed));
        app.add_plugins(direct_dial::FriendsPlugin::new(direct_dial::Config::from(
            self.peer_book_file.clone(),
        )));
        app.add_plugins(Self::lobby_plugins(discovery::Config {
            display_name: self.username.clone(),
            game: self.game.clone(),
            lease: self.lease,
            ..discovery::Config::default()
        }));
        app.add_plugins(
            replica_sync::SyncPlugin::<Canvas>::new(self.game.clone())
                .with_history_config(Some(self.history)),
        );
        app.add_plugins(peek::RoomPeekPlugin::<Canvas>::new(self.game.clone()));
        app.add_plugins(StarboardPlugin);
    }
}

impl StarboardApp {
    pub(crate) fn lobby_plugins(config: discovery::Config) -> PluginGroupBuilder {
        let lobby = LobbyRoomPlugins(config).build();
        lobby.disable::<ui::discovery::PreviewFriendsCardPlugin>()
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::lobby_room::{discovery, ui};

    use super::StarboardApp;

    #[test]
    fn test_usage() {
        let window = StarboardApp::primary_window("p1");
        assert_eq!(window.title, "starboard-p1");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn the_starboard_lobby_adds_the_ui_but_not_the_preview_friends_card() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StarboardApp::lobby_plugins(discovery::Config::default()));

        assert!(
            app.is_plugin_added::<ui::discovery::LobbyRoomUiPlugin>(),
            "the lobby UI itself stays on"
        );
        assert!(
            !app.is_plugin_added::<ui::discovery::PreviewFriendsCardPlugin>(),
            "Starboard draws friends on the board, so no RoomPreviewFriends card can spawn"
        );
    }
}
