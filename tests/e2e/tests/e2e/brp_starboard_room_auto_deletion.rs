use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::brp::{BrpNetwork, PEER_WINDOWS, StarboardPeer, require_real_network};
use crate::common::build_package_bin;

const HEARTBEATS_BEFORE_CRASH: Duration = Duration::from_secs(90);
const ROOM_AUTO_DELETION_EXPIRY_WINDOW: Duration = Duration::from_mins(12);

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display, the lavapipe ICD and a real network; run with --run-ignored all"]
async fn brp_starboard_room_auto_deletion() -> Result<(), String> {
    require_real_network("brp_starboard_room_auto_deletion")?;
    let net = BrpNetwork::prepare().await?;
    let bin = build_package_bin("e2e_apps", "starboard_e2e", "dev")?;
    let room = format!("room-{}", unix_secs());
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
    observer.exit().await
}
