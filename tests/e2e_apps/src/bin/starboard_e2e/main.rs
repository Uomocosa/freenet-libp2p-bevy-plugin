mod args;
mod transport_arg;
mod window_origin;

use clap::Parser;
use freenet_libp2p_bevy_plugin::identity;

use crate::args::Args;

#[tokio::main]
async fn main() -> bevy::app::AppExit {
    let args = Args::parse();
    Args::init_tracing();
    if let Err(error) = args.network.validate() {
        eprintln!("{error}");
        return bevy::app::AppExit::error();
    }
    let Ok((_node_guard, ws_port)) = examples_shared::node::start_embedded_freenet_node(
        args.freenet_node_dir.clone(),
        &args.network,
    )
    .await
    else {
        eprintln!("failed to start embedded freenet node");
        return bevy::app::AppExit::error();
    };
    let game = args.token.clone().map_or_else(
        || freenet_libp2p_bevy_plugin::game_id!(),
        identity::GameId::from,
    );
    args.run(ws_port, &game)
}
