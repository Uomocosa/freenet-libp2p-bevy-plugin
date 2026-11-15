mod args;

use clap::Parser;
use examples_shared as shared;
use freenet_libp2p_bevy_plugin::freenet_node;
use freenet_libp2p_bevy_plugin::identity;

#[tokio::main]
async fn main() -> bevy::app::AppExit {
    let args = args::Args::parse();
    args::Args::init_tracing();
    let network = match shared::example_network::network_args(args.network) {
        Ok(network) => network,
        Err(error) => {
            eprintln!("{error}");
            return bevy::app::AppExit::error();
        }
    };
    if let Err(error) = network.validate() {
        eprintln!("{error}");
        return bevy::app::AppExit::error();
    }
    let node = match freenet_node::EmbeddedNode::start(args.freenet_node_dir.clone()).await {
        Ok(node) => node,
        Err(error) => {
            eprintln!("failed to start embedded freenet node: {error}");
            return bevy::app::AppExit::error();
        }
    };
    let ws_port = node.websocket_port;
    let game = args.token.clone().map_or_else(
        || freenet_libp2p_bevy_plugin::game_id!(),
        identity::GameId::from,
    );
    args.run_starboard(ws_port, &game)
}
