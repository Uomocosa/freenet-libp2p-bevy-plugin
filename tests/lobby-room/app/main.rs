use clap::Parser;
use freenet_libp2p_bevy_plugin::freenet_node;
use freenet_libp2p_bevy_plugin::identity;
use freenet_libp2p_bevy_plugin::testing::Network;
use lobby_room_e2e::args::Args;

#[tokio::main]
async fn main() -> bevy::app::AppExit {
    let args = Args::parse();
    Args::init_tracing();
    if let Err(error) = args.network.validate() {
        eprintln!("{error}");
        return bevy::app::AppExit::error();
    }
    let (_node_dir, ws_port) =
        if matches!(args.network.network, Network::Fake | Network::DirectDial) {
            match freenet_node::NodeDir::try_from(args.freenet_node_dir.clone()) {
                Ok(node_dir) => (node_dir, 0),
                Err(error) => {
                    eprintln!("failed to prepare the freenet node folder: {error}");
                    return bevy::app::AppExit::error();
                }
            }
        } else {
            match freenet_node::EmbeddedNode::start(args.freenet_node_dir.clone()).await {
                Ok(node) => (node.node_dir, node.websocket_port),
                Err(error) => {
                    eprintln!("failed to start embedded freenet node: {error}");
                    return bevy::app::AppExit::error();
                }
            }
        };
    let game = args.token.clone().map_or_else(
        || freenet_libp2p_bevy_plugin::game_id!(),
        identity::GameId::from,
    );
    args.run(ws_port, &game)
}
