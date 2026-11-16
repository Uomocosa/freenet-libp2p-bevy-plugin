use freenet_libp2p_bevy_plugin::time::UnixTime;

use crate::brp::{BrpNetwork, PEER_WINDOWS, StarboardPeer};
use crate::common::build_package_bin;

const BAD_TOKEN: &str = "lobby-e2e/bad-token";

/// A peer on another lobby, or asking for a room that does not exist, never lists the room:
/// the join fails as the old `wait-join-failed` step did on a fetched lobby without the room.
fn expect_never_listed(peer: &str, reason: &str) -> Result<(), String> {
    if reason.contains("is not listed") {
        Ok(())
    } else {
        Err(format!(
            "{peer}: expected the room not listed, got {reason:?}"
        ))
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn starboard_bad_token() -> Result<(), String> {
    let net = BrpNetwork::prepare().await?;
    let mut recording = net.recording("starboard_bad_token");
    let bin = build_package_bin("starboard_e2e", "starboard_e2e", "dev")?;
    let room = format!("brp-{}", UnixTime::now().as_secs());
    let creator = StarboardPeer::launch(&bin, "creator", &net, PEER_WINDOWS[0]).await?;
    creator.create_room(&room).await?;
    let watcher = StarboardPeer::launch(&bin, "watcher", &net, PEER_WINDOWS[1]).await?;
    watcher.wait_until_room_listed(&room).await?;
    let bad_token =
        StarboardPeer::launch_with_token(&bin, "bad_token", &net, PEER_WINDOWS[2], BAD_TOKEN)
            .await?;
    expect_never_listed(
        "bad_token",
        &bad_token.join_room_expecting_failure(&room).await?,
    )?;
    let missing_room = format!("no-such-room-{room}");
    let missing = StarboardPeer::launch(&bin, "missing_room", &net, PEER_WINDOWS[3]).await?;
    expect_never_listed(
        "missing_room",
        &missing.join_room_expecting_failure(&missing_room).await?,
    )?;
    bad_token.exit().await?;
    missing.exit().await?;
    watcher.exit().await?;
    creator.exit().await?;
    recording.mark_passed();
    Ok(())
}
