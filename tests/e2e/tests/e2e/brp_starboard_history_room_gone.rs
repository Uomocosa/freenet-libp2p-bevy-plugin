use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::brp::{BrpNetwork, PEER_WINDOWS, StarboardPeer, Stroke};
use crate::common::build_package_bin;

const STROKES: &[Stroke] = &[
    Stroke(&[(200.0, 300.0), (260.0, 300.0)]),
    Stroke(&[(230.0, 309.0), (290.0, 309.0)]),
    Stroke(&[(260.0, 318.0), (320.0, 318.0)]),
    Stroke(&[(290.0, 327.0), (350.0, 327.0)]),
    Stroke(&[(320.0, 336.0), (380.0, 336.0)]),
    Stroke(&[(350.0, 345.0), (410.0, 345.0)]),
];
const ERASE_STROKES: &[Stroke] = &[
    Stroke(&[(200.0, 300.0), (260.0, 300.0)]),
    Stroke(&[(230.0, 309.0), (290.0, 309.0)]),
    Stroke(&[(260.0, 318.0), (320.0, 318.0)]),
];
const OTHERS_IN_TWO: usize = 1;
const QUIET: Duration = Duration::from_secs(3);
const HOLDERS_SETTLE: Duration = Duration::from_secs(45);

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn brp_starboard_history_room_gone() -> Result<(), String> {
    let net = BrpNetwork::prepare().await?;
    let bin = build_package_bin("e2e_apps", "starboard_e2e", "dev")?;
    let room = format!("brp-{}", unix_secs());
    let painter = StarboardPeer::launch(&bin, "painter", &net, PEER_WINDOWS[0]).await?;
    painter.create_room(&room).await?;
    let helper = StarboardPeer::launch(&bin, "helper", &net, PEER_WINDOWS[1]).await?;
    helper.join_room(&room).await?;
    let watcher = StarboardPeer::launch(&bin, "watcher", &net, PEER_WINDOWS[2]).await?;
    watcher.wait_until_room_listed(&room).await?;
    painter
        .wait_until_room_has_at_least_other_members(OTHERS_IN_TWO)
        .await?;
    helper
        .wait_until_room_has_at_least_other_members(OTHERS_IN_TWO)
        .await?;
    painter.paint_all(STROKES).await?;
    helper
        .wait_until_canvas_has_content(net.deadlines().scaled(Duration::from_secs(60)))
        .await?;
    helper.erase_all(ERASE_STROKES).await?;
    painter.wait_until_canvas_stops_changing(QUIET).await?;
    helper.wait_until_canvas_stops_changing(QUIET).await?;
    tokio::time::sleep(HOLDERS_SETTLE).await;
    painter.exit().await?;
    helper.exit().await?;
    watcher.wait_until_room_gone(&room).await?;
    let late_joiner = StarboardPeer::launch(&bin, "late_joiner", &net, PEER_WINDOWS[3]).await?;
    let reason = late_joiner.join_room_expecting_failure(&room).await?;
    if !reason.contains("is not listed") && !reason.contains("no longer exists") {
        return Err(format!(
            "late_joiner: expected the room gone, got {reason:?}"
        ));
    }
    late_joiner.exit().await?;
    watcher.exit().await
}
