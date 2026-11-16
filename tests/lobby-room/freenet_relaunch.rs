use std::time::Duration;

use crate::brp::{BrpNetwork, LobbyRoomPeer, LobbySetup, PEER_WINDOWS};
use crate::common::build_package_bin;
use freenet_libp2p_bevy_plugin::testing::Network;

const RING_DEADLINE: Duration = Duration::from_secs(120);
const LOBBY_READY_DEADLINE: Duration = Duration::from_secs(60);
const DEFAULT_GAP_SECS: u64 = 10;

fn relaunch_gap() -> Duration {
    std::env::var("E2E_RELAUNCH_GAP_SECS")
        .ok()
        .and_then(|raw| raw.parse().ok())
        .map_or(Duration::from_secs(DEFAULT_GAP_SECS), Duration::from_secs)
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display, the lavapipe ICD and a real network; run with --run-ignored all"]
async fn freenet_relaunch() -> Result<(), String> {
    Network::from_env().require_real("freenet_relaunch")?;
    let net = BrpNetwork::try_from_network_setup(Network::from_env(), LobbySetup::Absent).await?;
    let bin = build_package_bin("lobby_room_e2e", "lobby_room_e2e", "dev")?;
    let cold = LobbyRoomPeer::launch(&bin, "relaunch", &net, PEER_WINDOWS[0]).await?;
    cold.wait_until_ring_joined(RING_DEADLINE).await?;
    cold.wait_until_lobby_fetched(LOBBY_READY_DEADLINE).await?;
    cold.exit().await?;
    tokio::time::sleep(relaunch_gap()).await;
    let warm = LobbyRoomPeer::launch(&bin, "relaunch", &net, PEER_WINDOWS[0]).await?;
    warm.wait_until_ring_joined(RING_DEADLINE).await?;
    warm.wait_until_lobby_fetched(LOBBY_READY_DEADLINE).await?;
    warm.exit().await
}
