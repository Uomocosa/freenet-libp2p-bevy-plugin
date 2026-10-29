#[path = "mod.rs"]
mod starboard;

#[path = "../shared/mod.rs"]
mod shared;

mod args;
mod parse_stroke;
mod scaled_timings;
mod script;
mod script_runner;
mod status;

use bevy::prelude::*;
use clap::Parser;
use freenet_libp2p_bevy_plugin::{discovery, identity, p2p, plugin, room_sync};

#[tokio::main]
async fn main() -> bevy::app::AppExit {
    let args = args::Args::parse();
    init_tracing();
    let Ok((_node_guard, ws_port)) =
        shared::node::start_embedded_freenet_node(args.freenet_node_dir.clone()).await
    else {
        eprintln!("failed to start embedded freenet node");
        return bevy::app::AppExit::error();
    };
    let game = args.token.clone().map_or_else(
        || freenet_libp2p_bevy_plugin::game_id!(),
        identity::GameId::from,
    );
    let time_scale = time_scale();
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: format!("starboard-{}", args.username),
            resolution: (960, 540).into(),
            ..default()
        }),
        ..default()
    }));
    app.add_systems(Startup, spawn_camera);
    app.insert_resource(discovery::FreenetPort(ws_port));
    app.insert_resource(shared::status::LocalUser(identity::Username::from(
        args.username.clone(),
    )));
    app.insert_resource(shared::intent::Intent::new(None));
    app.insert_resource(shared::intent::PendingAction::default());
    app.add_plugins(plugin::P2PPlugin(
        plugin::Config::<starboard::room::CursorUpdate>::new(
            args::transport_mode(args.transport),
            p2p::MdnsMode::Disabled,
        )
        .with_game(game.clone()),
    ));
    app.add_plugins(discovery::Plugins(discovery::Config {
        display_name: identity::Username::from(args.username.clone()),
        game: game.clone(),
        lease: scaled_timings::scaled_lease_timings(time_scale),
        ..discovery::Config::default()
    }));
    app.add_plugins(
        room_sync::SyncPlugin::<starboard::canvas::Canvas>::new(game)
            .with_history_config(Some(scaled_timings::scaled_history_config(time_scale))),
    );
    app.add_plugins(starboard::app::StarboardPlugin);
    app.add_systems(
        Update,
        shared::intent::script_input::<
            discovery::ui::RoomNameInput,
            discovery::ui::CreateRoomButton,
            discovery::ui::RoomButton,
        >,
    );
    if let Some(steps) = args.script.clone() {
        app.insert_resource(script_runner::StarboardScriptRunner::new(steps));
        app.add_systems(
            Update,
            script_runner::run_script
                .after(discovery::bevy_systems::drain_status)
                .before(
                    shared::intent::script_input::<
                        discovery::ui::RoomNameInput,
                        discovery::ui::CreateRoomButton,
                        discovery::ui::RoomButton,
                    >,
                ),
        );
    }
    app.add_systems(
        Update,
        (
            shared::intent::log_joined,
            shared::status::log_tick,
            shared::status::log_mesh,
            status::log_canvas_hash,
        ),
    );
    app.run()
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// needed helper: the run's time scale, from E2E_TIME_SCALE
fn time_scale() -> f64 {
    std::env::var("E2E_TIME_SCALE")
        .ok()
        .and_then(|raw| raw.parse::<f64>().ok())
        .filter(|scale| scale.is_finite() && *scale > 0.0)
        .unwrap_or(1.0)
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
