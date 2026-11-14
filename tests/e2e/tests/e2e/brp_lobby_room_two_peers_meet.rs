use std::time::{SystemTime, UNIX_EPOCH};

use freenet_libp2p_bevy_plugin::lobby_room::testing::LobbyKit;
use freenet_libp2p_bevy_plugin::testing::FakeNetworkServer;

use crate::brp::LobbyRoomPeer;
use crate::build_game::build_package_bin;

const ALICE_WINDOW: (i32, i32) = (0, 0);
const BOB_WINDOW: (i32, i32) = (481, 0);

fn unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos())
}

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn brp_lobby_room_two_peers_meet() -> Result<(), String> {
    let orchestrator = FakeNetworkServer::start_with(Box::new(LobbyKit::default())).await?;
    let bin = build_package_bin("e2e_apps", "lobby_room_e2e", "dev")?;
    let token = format!("brp-{}", unix_nanos());
    let room = format!("brp-{}", unix_secs());
    let alice = LobbyRoomPeer::launch(&bin, "alice", &token, *orchestrator, ALICE_WINDOW).await?;
    let bob = LobbyRoomPeer::launch(&bin, "bob", &token, *orchestrator, BOB_WINDOW).await?;
    alice.create_room(&room).await?;
    bob.join_room(&room).await?;
    if let Err(error) = alice.wait_until_room_has_other_members(1).await {
        return Err(format!("{error}\n{}", bob.describe().await));
    }
    bob.wait_until_room_has_other_members(1).await?;
    alice.wait_until_hears_a_peer().await?;
    bob.wait_until_hears_a_peer().await?;
    alice.exit().await?;
    bob.exit().await
}
