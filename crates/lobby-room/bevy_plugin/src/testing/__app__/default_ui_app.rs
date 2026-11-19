use std::time::Duration;

use bevy::input::InputPlugin;
use bevy::prelude::{App, MinimalPlugins};
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

use crate::direct_dial as core_direct_dial;
use crate::discovery;
use crate::identity;
use crate::p2p;
use crate::plugin;
use crate::testing;
use crate::ui;

pub fn default_ui_app(network: &testing::NetworkArgs) -> App {
    let game = identity::GameId::from("ui-flow");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(InputPlugin);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        50,
    )));
    app.insert_resource(discovery::FreenetPort(0));
    app.add_plugins(testing::FakeNetworkPlugin::<u32>::from(network));
    app.add_plugins(plugin::P2PPlugin(
        plugin::Config::<u32>::new(p2p::TransportMode::Both, p2p::MdnsMode::Disabled)
            .with_game(game.clone()),
    ));
    app.add_plugins(core_direct_dial::FriendsPlugin::new(
        core_direct_dial::Config::from(None),
    ));
    app.add_plugins(discovery::Plugin::new(discovery::Config {
        display_name: identity::Username::from("tester"),
        game,
        ..discovery::Config::default()
    }));
    app.world_mut().insert_resource(
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("ui-flow"))
            .in_room(testing::joined_room(
                "ui-room",
                0,
                discovery::JoinPhase::Ready,
                Vec::new(),
            ))
            .build(),
    );
    app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
    app.add_plugins(ui::direct_dial::FriendsUiPlugin);
    app.finish();
    app
}

// no test_usage necessary
