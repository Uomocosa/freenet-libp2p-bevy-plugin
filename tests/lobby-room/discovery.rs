use freenet_libp2p_bevy_plugin::time::UnixTime;
use std::time::Duration;

use crate::brp::{BrpNetwork, LobbyRoomPeer, Network, PEER_WINDOWS};
use crate::common::build_package_bin;

const RING_DEADLINE: Duration = Duration::from_secs(120);

fn peer_count() -> usize {
    std::env::var("E2E_PEERS")
        .ok()
        .and_then(|raw| raw.parse().ok())
        .unwrap_or(3)
        .clamp(2, 5)
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display, the lavapipe ICD and a real network; run with --run-ignored all"]
async fn lobby_room_discovery() -> Result<(), String> {
    Network::from_env().require_real("lobby_room_discovery")?;
    let net = BrpNetwork::prepare().await?;
    let bin = build_package_bin("lobby_room_e2e", "lobby_room_e2e", "dev")?;
    let room = format!("brp-discover-{}", UnixTime::now().as_secs());
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
