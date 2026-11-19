use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::path::PathBuf;

use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;

use crate::freenet_node;

pub async fn start(
    kept_node_dir: Option<PathBuf>,
) -> Result<freenet_node::EmbeddedNode, freenet_node::Error> {
    let node_dir =
        freenet_node::NodeDir::try_from(kept_node_dir).map_err(freenet_node::Error::NodeDir)?;
    let base = node_dir.path().to_path_buf();
    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .map_err(freenet_node::Error::Bind)?;
    let websocket_port = listener
        .local_addr()
        .map_err(freenet_node::Error::Bind)?
        .port();
    let websocket_config = WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: websocket_port,
        ..Default::default()
    };
    let clients = serve_client_api_with_listener(websocket_config, listener)
        .await
        .map_err(freenet_node::Error::ClientApi)?;
    let network_port = match node_dir {
        freenet_node::NodeDir::Kept(_) => None,
        freenet_node::NodeDir::Temporary(_) => Some(freenet_node::EmbeddedNode::free_udp_port()?),
    };
    let args = ConfigArgs {
        mode: Some(OperationMode::Network),
        network_api: NetworkArgs {
            is_gateway: false,
            network_port,
            ..Default::default()
        },
        config_paths: ConfigPathsArgs {
            config_dir: Some(base.clone()),
            data_dir: Some(base.clone()),
            log_dir: Some(base),
        },
        ..Default::default()
    };
    let config = args.build().await.map_err(freenet_node::Error::Config)?;
    let node_config = NodeConfig::new(config)
        .await
        .map_err(freenet_node::Error::Config)?;
    let node = node_config
        .build(clients)
        .await
        .map_err(freenet_node::Error::Build)?;
    tokio::spawn(async move {
        if let Err(error) = Box::pin(run_network_node(node)).await {
            tracing::error!(error = %error, "embedded freenet node exited");
        }
    });
    tracing::info!("freenet node started ws_port={websocket_port}");
    Ok(freenet_node::EmbeddedNode {
        node_dir,
        websocket_port,
    })
}

// no test_usage necessary
