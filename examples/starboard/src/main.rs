mod args;
mod friend_intent;
mod result;
mod scaled_timings;
mod script;
mod script_runner;
mod scripted_starboard_step;
mod status;
mod time_scale;

use clap::Parser;
use examples_shared as shared;
use freenet_libp2p_bevy_plugin::identity;
use starboard::starboard;

#[tokio::main]
async fn main() -> bevy::app::AppExit {
    let args = args::Args::parse();
    args::Args::init_tracing();
    if let Err(error) = args.network.validate() {
        eprintln!("{error}");
        return bevy::app::AppExit::error();
    }
    let Ok((_node_guard, ws_port)) =
        shared::node::start_embedded_freenet_node(args.freenet_node_dir.clone(), &args.network)
            .await
    else {
        eprintln!("failed to start embedded freenet node");
        return bevy::app::AppExit::error();
    };
    let game = args.token.clone().map_or_else(
        || freenet_libp2p_bevy_plugin::game_id!(),
        identity::GameId::from,
    );
    args.run_starboard(ws_port, &game, time_scale::Scale::from_env())
}
