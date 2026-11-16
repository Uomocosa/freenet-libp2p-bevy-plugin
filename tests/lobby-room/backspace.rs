use freenet_libp2p_bevy_plugin::time::UnixTime;

use crate::brp::{BrpNetwork, LobbyRoomPeer};
use crate::common::build_package_bin;

const ALICE_WINDOW: (i32, i32) = (0, 0);

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn lobby_room_backspace() -> Result<(), String> {
    let net = BrpNetwork::prepare().await?;
    let mut recording = net.recording("lobby_room_backspace");
    let bin = build_package_bin("lobby_room_e2e", "lobby_room_e2e", "dev")?;
    let room = format!("brp-{}", UnixTime::now().as_secs());
    let alice = LobbyRoomPeer::launch(&bin, "alice", &net, ALICE_WINDOW).await?;
    alice.create_room_after_backspace(&room).await?;
    let joined = alice.lobby_status().await?.joined_room;
    if joined.as_deref() != Some(room.as_str()) {
        return Err(format!(
            "joined {joined:?} after Backspace, expected {room:?}"
        ));
    }
    alice.exit().await?;
    recording.mark_passed();
    Ok(())
}
