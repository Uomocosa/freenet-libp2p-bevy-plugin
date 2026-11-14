use std::path::PathBuf;

use bevy::prelude::*;
use bevy::window::Window;
use freenet_libp2p_bevy_plugin::lobby_room::{LobbyRoomPlugins, discovery};
use freenet_libp2p_bevy_plugin::p2p::{MdnsMode, TransportMode};
use freenet_libp2p_bevy_plugin::{direct_dial, identity, plugin, room_sync};

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
    pub history: room_sync::HistoryConfig,
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
        app.add_plugins(LobbyRoomPlugins(discovery::Config {
            display_name: self.username.clone(),
            game: self.game.clone(),
            lease: self.lease,
            ..discovery::Config::default()
        }));
        app.add_plugins(
            room_sync::SyncPlugin::<Canvas>::new(self.game.clone())
                .with_history_config(Some(self.history)),
        );
        app.add_plugins(StarboardPlugin);
    }
}

#[cfg(test)]
mod tests {
    use super::StarboardApp;

    #[test]
    fn test_usage() {
        let window = StarboardApp::primary_window("p1");
        assert_eq!(window.title, "starboard-p1");
    }
}
