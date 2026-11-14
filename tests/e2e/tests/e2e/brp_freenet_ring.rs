use std::time::Duration;

use crate::brp::{BrpNetwork, LobbyRoomPeer, PEER_WINDOWS, require_real_network};
use crate::common::build_package_bin;

const RING_DEADLINE: Duration = Duration::from_secs(120);
const LOBBY_READY_DEADLINE: Duration = Duration::from_secs(60);

fn peer_count() -> usize {
    std::env::var("E2E_PEERS")
        .ok()
        .and_then(|raw| raw.parse().ok())
        .unwrap_or(3)
        .clamp(1, 5)
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display, the lavapipe ICD and a real network; run with --run-ignored all"]
async fn brp_freenet_ring() -> Result<(), String> {
    require_real_network("brp_freenet_ring")?;
    let net = BrpNetwork::prepare().await?;
    let bin = build_package_bin("e2e_apps", "lobby_room_e2e", "dev")?;
    let mut peers = Vec::new();
    for index in 0..peer_count() {
        let name = format!("p{}", index.saturating_add(1));
        let window = PEER_WINDOWS
            .iter()
            .cycle()
            .nth(index)
            .copied()
            .unwrap_or((0, 0));
        peers.push(LobbyRoomPeer::launch(&bin, &name, &net, window).await?);
    }
    for peer in &peers {
        peer.wait_until_ring_joined(RING_DEADLINE).await?;
        peer.wait_until_lobby_fetched(LOBBY_READY_DEADLINE).await?;
    }
    for peer in peers {
        peer.exit().await?;
    }
    Ok(())
}
