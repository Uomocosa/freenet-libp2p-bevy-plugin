use std::net::{IpAddr, Ipv4Addr, TcpListener, UdpSocket};

use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;

pub async fn bootstrap_node() -> Result<(tempfile::TempDir, u16), String> {
    let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
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
            config_dir: Some(tmp.path().to_path_buf()),
            data_dir: Some(tmp.path().to_path_buf()),
            log_dir: Some(tmp.path().to_path_buf()),
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
    tracing::info!("embedded freenet node started ws_port={ws_port}");
    Ok((tmp, ws_port))
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
