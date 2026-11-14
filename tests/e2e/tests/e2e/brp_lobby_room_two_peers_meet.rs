use std::time::{SystemTime, UNIX_EPOCH};

use crate::brp::{BrpNetwork, LobbyRoomPeer};
use crate::common::build_package_bin;

const ALICE_WINDOW: (i32, i32) = (0, 0);
const BOB_WINDOW: (i32, i32) = (481, 0);

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn brp_lobby_room_two_peers_meet() -> Result<(), String> {
    let net = BrpNetwork::prepare().await?;
    let bin = build_package_bin("e2e_apps", "lobby_room_e2e", "dev")?;
    let room = format!("brp-{}", unix_secs());
    let alice = LobbyRoomPeer::launch(&bin, "alice", &net, ALICE_WINDOW).await?;
    let bob = LobbyRoomPeer::launch(&bin, "bob", &net, BOB_WINDOW).await?;
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
