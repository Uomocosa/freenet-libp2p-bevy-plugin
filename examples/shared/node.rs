use std::net::{IpAddr, Ipv4Addr, TcpListener, UdpSocket};
use std::path::PathBuf;

use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;

use super::freenet_node_dir::FreenetNodeDir;

pub async fn start_embedded_freenet_node(
    freenet_node_dir: Option<PathBuf>,
) -> Result<(FreenetNodeDir, u16), String> {
    let node_dir = if let Some(path) = freenet_node_dir {
        std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
        FreenetNodeDir::Kept(path)
    } else {
        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
        FreenetNodeDir::Temporary(tmp)
    };
    let base = match &node_dir {
        FreenetNodeDir::Temporary(tmp) => tmp.path().to_path_buf(),
        FreenetNodeDir::Kept(path) => path.clone(),
    };
    let listener =
        TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0)).map_err(|e| e.to_string())?;
    let ws_port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let ws_config = WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: ws_port,
        ..Default::default()
    };
    let clients = serve_client_api_with_listener(ws_config, listener)
        .await
        .map_err(|e| e.to_string())?;
    let args = ConfigArgs {
        mode: Some(OperationMode::Network),
        network_api: NetworkArgs {
            is_gateway: false,
            network_port: Some(free_udp_port()?),
            ..Default::default()
        },
        config_paths: ConfigPathsArgs {
            config_dir: Some(base.clone()),
            data_dir: Some(base.clone()),
            log_dir: Some(base.clone()),
        },
        ..Default::default()
    };
    let config = args.build().await.map_err(|e| e.to_string())?;
    let node_config = NodeConfig::new(config).await.map_err(|e| e.to_string())?;
    let node = node_config
        .build(clients)
        .await
        .map_err(|e| e.to_string())?;
    tokio::spawn(async move {
        if let Err(e) = run_network_node(node).await {
            tracing::error!(error = %e, "embedded freenet node exited");
        }
    });
    tracing::info!("lobby freenet node started ws_port={ws_port}");
    Ok((node_dir, ws_port))
}

// needed helper: allocates a free UDP port so parallel instances never collide
fn free_udp_port() -> Result<u16, String> {
    let socket =
        UdpSocket::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0)).map_err(|e| e.to_string())?;
    socket
        .local_addr()
        .map_err(|e| e.to_string())
        .map(|addr| addr.port())
}
