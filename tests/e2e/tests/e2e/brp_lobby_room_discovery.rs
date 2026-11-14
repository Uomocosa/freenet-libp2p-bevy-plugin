use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::brp::{BrpNetwork, LobbyRoomPeer, PEER_WINDOWS, require_real_network};
use crate::common::build_package_bin;

const RING_DEADLINE: Duration = Duration::from_secs(120);

fn peer_count() -> usize {
    std::env::var("E2E_PEERS")
        .ok()
        .and_then(|raw| raw.parse().ok())
        .unwrap_or(3)
        .clamp(2, 5)
}

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display, the lavapipe ICD and a real network; run with --run-ignored all"]
async fn brp_lobby_room_discovery() -> Result<(), String> {
    require_real_network("brp_lobby_room_discovery")?;
    let net = BrpNetwork::prepare().await?;
    let bin = build_package_bin("e2e_apps", "lobby_room_e2e", "dev")?;
    let room = format!("brp-discover-{}", unix_secs());
    let creator_window = PEER_WINDOWS.first().copied().unwrap_or((0, 0));
    let creator = LobbyRoomPeer::launch(&bin, "p1", &net, creator_window).await?;
    creator.create_room(&room).await?;
    creator.wait_until_ring_joined(RING_DEADLINE).await?;
    let mut joiners = Vec::new();
    for index in 1..peer_count() {
        let name = format!("p{}", index.saturating_add(1));
        let window = PEER_WINDOWS
            .iter()
            .cycle()
            .nth(index)
            .copied()
            .unwrap_or((0, 0));
        let joiner = LobbyRoomPeer::launch(&bin, &name, &net, window).await?;
        joiner
            .wait_until_room_listed(&room)
            .await
            .map_err(|error| format!("{name}: {error}"))?;
        joiner.wait_until_ring_joined(RING_DEADLINE).await?;
        joiners.push(joiner);
    }
    for joiner in joiners {
        joiner.exit().await?;
    }
    creator.exit().await
}
