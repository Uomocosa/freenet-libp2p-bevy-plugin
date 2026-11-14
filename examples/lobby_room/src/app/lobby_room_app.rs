use std::path::PathBuf;

use bevy::prelude::*;
use bevy::window::Window;
use examples_shared::status;
use freenet_libp2p_bevy_plugin::lobby_room::{LobbyRoomPlugins, discovery, ui as plugin_ui};
use freenet_libp2p_bevy_plugin::p2p::{MdnsMode, TransportMode};
use freenet_libp2p_bevy_plugin::{direct_dial, identity, plugin};

use crate::app::UiChoice;
use crate::{overlay, ping, ui};

pub struct LobbyRoomApp {
    pub username: identity::Username,
    pub game: identity::GameId,
    pub transport: TransportMode,
    pub peer_book_file: Option<PathBuf>,
    pub freenet_port: u16,
    pub room_ping: bool,
    pub ui: UiChoice,
}

impl LobbyRoomApp {
    #[must_use]
    pub fn primary_window(username: &str) -> Window {
        Window {
            title: format!("lobby-{username}"),
            resolution: (960, 540).into(),
            ..default()
        }
    }
}

impl Plugin for LobbyRoomApp {
    fn build(&self, app: &mut App) {
        app.insert_resource(discovery::FreenetPort(self.freenet_port));
        app.insert_resource(status::LocalUser(self.username.clone()));
        app.insert_resource(ping::IsRoomPingEnabled(self.room_ping));
        app.init_resource::<ping::RoomMessageSenders>();
        app.add_plugins(plugin::P2PPlugin(
            plugin::Config::<ping::PingPayload>::new(self.transport, MdnsMode::Disabled)
                .with_game(self.game.clone()),
        ));
        app.add_plugins(direct_dial::FriendsPlugin::new(direct_dial::Config::from(
            self.peer_book_file.clone(),
        )));
        let lobby_plugins = LobbyRoomPlugins(discovery::Config {
            display_name: self.username.clone(),
            game: self.game.clone(),
            ..discovery::Config::default()
        });
        match self.ui {
            UiChoice::Default => {
                app.add_plugins(lobby_plugins);
            }
            UiChoice::Custom => {
                app.add_plugins(
                    lobby_plugins
                        .build()
                        .disable::<plugin_ui::discovery::LobbyRoomUiPlugin>()
                        .disable::<plugin_ui::direct_dial::FriendsUiPlugin>(),
                );
                app.add_plugins(ui::LobbyUi);
            }
        }
        app.add_plugins(overlay::Overlay);
        app.add_systems(Update, (ping::publish_room_ping, ping::log_room_messages));
    }
}

#[cfg(test)]
mod tests {
    use super::LobbyRoomApp;

    #[test]
    fn test_usage() {
        let window = LobbyRoomApp::primary_window("p1");
        assert_eq!(window.title, "lobby-p1");
    }
}
