#[path = "../shared/mod.rs"]
mod shared;

mod args;
mod journey;
mod overlay;
mod ping;
mod script_runner;
mod ui;

use bevy::prelude::*;
use clap::Parser;
use freenet_libp2p_bevy_plugin::{discovery, identity, p2p, plugin};

#[tokio::main]
async fn main() -> bevy::app::AppExit {
    let args = args::Args::parse();
    init_tracing();
    let Ok((_node_guard, ws_port)) = shared::node::bootstrap_node().await else {
        eprintln!("failed to start embedded freenet node");
        return bevy::app::AppExit::error();
    };
    let transport = args::transport_mode(args.transport);
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: format!("lobby-{}", args.username),
            resolution: (960, 540).into(),
            ..default()
        }),
        ..default()
    }));
    app.insert_resource(discovery::FreenetPort(ws_port));
    let username = identity::Username::from(args.username.clone());
    app.insert_resource(shared::status::LocalUser(username.clone()));
    app.insert_resource(ping::IsRoomPingEnabled(args.is_room_ping_enabled));
    app.insert_resource(shared::intent::Intent::new(None));
    app.insert_resource(shared::intent::PendingAction::default());
    app.init_resource::<ping::RoomMessageSenders>();
    if let Some(steps) = args.script.clone() {
        app.insert_resource(script_runner::LobbyScriptRunner::new(steps));
        app.add_systems(
            Update,
            script_runner::run_script
                .after(discovery::bevy_systems::drain_status)
                .after(ping::log_room_messages),
        );
    }
    let game = args.token.clone().map_or_else(
        || freenet_libp2p_bevy_plugin::game_id!(),
        identity::GameId::from,
    );
    app.add_plugins(plugin::P2PPlugin(
        plugin::Config::<ping::PingPayload>::new(transport, p2p::MdnsMode::Disabled)
            .with_game(game.clone()),
    ));
    let discovery_plugins = discovery::Plugins(discovery::Config {
        display_name: username,
        game,
        ..discovery::Config::default()
    });
    // scripted input runs through Bevy's real typing/clicking paths
    match args.ui {
        args::UiArg::Default => {
            app.add_plugins(discovery_plugins);
            app.add_systems(
                Update,
                shared::intent::script_input::<
                    discovery::ui::RoomNameInput,
                    discovery::ui::CreateRoomButton,
                    discovery::ui::RoomButton,
                >,
            );
        }
        args::UiArg::Custom => {
            app.add_plugins(
                discovery_plugins
                    .build()
                    .disable::<discovery::ui::DefaultUiPlugin>(),
            );
            app.add_plugins(ui::LobbyUi);
            app.add_systems(
                Update,
                shared::intent::script_input::<ui::NameField, ui::NewRoomButton, ui::JoinButton>,
            );
        }
    }
    app.add_plugins(overlay::Overlay);
    app.add_systems(
        Update,
        (
            shared::intent::log_joined,
            shared::status::log_tick,
            shared::status::log_mesh,
            ping::publish_room_ping,
            ping::log_room_messages,
        ),
    );
    app.run()
}

// needed helper: stdout tracing so the run log captures every marker line
fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .init();
}
