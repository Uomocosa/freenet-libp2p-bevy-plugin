use freenet_libp2p_bevy_plugin::time::UnixTime;
use std::time::Duration;

use crate::brp::{BrpNetwork, Network, PEER_WINDOWS, StarboardPeer};
use crate::common::build_package_bin;

const HEARTBEATS_BEFORE_CRASH: Duration = Duration::from_secs(90);
const ROOM_AUTO_DELETION_EXPIRY_WINDOW: Duration = Duration::from_mins(12);

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display, the lavapipe ICD and a real network; run with --run-ignored all"]
async fn starboard_room_auto_deletion() -> Result<(), String> {
    Network::from_env().require_real("starboard_room_auto_deletion")?;
    let net = BrpNetwork::prepare().await?;
    let mut recording = net.recording("starboard_room_auto_deletion");
    let bin = build_package_bin("starboard_e2e", "starboard_e2e", "dev")?;
    let room = format!("room-{}", UnixTime::now().as_secs());
    let mut creator = StarboardPeer::launch(&bin, "p1", &net, PEER_WINDOWS[0]).await?;
    creator.create_room(&room).await?;
    let observer = StarboardPeer::launch(&bin, "p2", &net, PEER_WINDOWS[1]).await?;
    observer.wait_until_room_listed(&room).await?;
    tokio::time::sleep(HEARTBEATS_BEFORE_CRASH).await;
    creator.kill()?;
    observer
        .wait_until_room_gone_within(&room, ROOM_AUTO_DELETION_EXPIRY_WINDOW)
        .await?;
    let late_observer = StarboardPeer::launch(&bin, "p3", &net, PEER_WINDOWS[2]).await?;
    late_observer
        .wait_until_room_gone_within(&room, ROOM_AUTO_DELETION_EXPIRY_WINDOW)
        .await?;
    late_observer.exit().await?;
    observer.exit().await?;
    recording.mark_passed();
    Ok(())
}
