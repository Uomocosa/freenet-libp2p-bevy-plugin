use std::time::{SystemTime, UNIX_EPOCH};

use crate::brp::{BrpNetwork, LobbyRoomPeer};
use crate::common::build_package_bin;

const ALICE_WINDOW: (i32, i32) = (0, 0);

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn brp_lobby_room_backspace() -> Result<(), String> {
    let net = BrpNetwork::prepare().await?;
    let bin = build_package_bin("e2e_apps", "lobby_room_e2e", "dev")?;
    let room = format!("brp-{}", unix_secs());
    let alice = LobbyRoomPeer::launch(&bin, "alice", &net, ALICE_WINDOW).await?;
    alice.create_room_after_backspace(&room).await?;
    let joined = alice.lobby_status().await?.joined_room;
    if joined.as_deref() != Some(room.as_str()) {
        return Err(format!(
            "joined {joined:?} after Backspace, expected {room:?}"
        ));
    }
    alice.exit().await
}
