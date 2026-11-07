use crate::testing;

pub async fn spawn_due_peers(
    orchestrator: &testing::Orchestrator,
    running: &mut [testing::RunningPeer],
    dir: &std::path::Path,
    bin: &std::path::Path,
    room: &str,
    scripts: &[String],
    token: &str,
    transport: &str,
    orchestrator_port: Option<u16>,
) {
    const SPAWN_GAP: std::time::Duration = std::time::Duration::from_millis(300);
    const DEFAULT_RUST_LOG: &str = "info,lobby_room=debug,starboard=debug,p2p=debug,discovery=debug,freenet_libp2p_bevy_plugin=debug";
    let live = orchestrator.live_peer_addresses(running);
    for index in 0..orchestrator.peers.len() {
        let Some((script, log)) =
            orchestrator.prepare_peer_spawn(index, room, scripts, token, running, &live, dir)
        else {
            continue;
        };
        let Some(peer) = orchestrator.peers.get(index) else {
            continue;
        };
        let Ok(log_file) = std::fs::File::create(&log) else {
            continue;
        };
        let Ok(err_file) = log_file.try_clone() else {
            continue;
        };
        let peer_token = orchestrator.peer_token(index, token);
        let mut args = vec![
            "--username".to_string(),
            peer.name.clone(),
            "--token".to_string(),
            peer_token,
            "--transport".to_string(),
            transport.to_string(),
            "--script".to_string(),
            script.clone(),
            "--network".to_string(),
            orchestrator.network_arg(),
        ];
        if let Some(port) = orchestrator_port {
            args.push("--orchestrator-port".to_string());
            args.push(port.to_string());
        }
        let node_dir = dir.join(format!("{}-freenet", peer.name));
        args.push("--freenet-node-dir".to_string());
        args.push(node_dir.display().to_string());
        let identity_file = orchestrator.peer_identity_file(token, &peer.name);
        args.push("--identity-file".to_string());
        args.push(identity_file.display().to_string());
        if matches!(orchestrator.network, testing::Network::DirectDial) {
            let book_file = orchestrator.peer_book_file(token, &peer.name);
            args.push("--peer-book-file".to_string());
            args.push(book_file.display().to_string());
        }
        let result_path = dir.join(&peer.name).join("result.json");
        args.push("--result-json".to_string());
        args.push(result_path.display().to_string());
        let mut rust_log =
            std::env::var("LOBBY_RUST_LOG").unwrap_or_else(|_| DEFAULT_RUST_LOG.to_string());
        if !rust_log.contains("room_sync") {
            rust_log.push(',');
            rust_log.push_str("room_sync=info");
        }
        let scale = orchestrator.time_scale();
        let child = std::process::Command::new(bin)
            .args(&args)
            .env("RUST_LOG", rust_log)
            .env("E2E_TIME_SCALE", scale.to_string())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::from(log_file))
            .stderr(std::process::Stdio::from(err_file))
            .spawn();
        if let Some(slot) = running.get_mut(index) {
            slot.child = child.ok();
            slot.log = log;
        }
        tokio::time::sleep(SPAWN_GAP).await;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(super::spawn_due_peers);
    }
}
