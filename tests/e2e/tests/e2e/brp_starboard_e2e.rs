use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::brp::{BrpNetwork, PEER_WINDOWS, StarboardPeer, Stroke, expect_canvases_converge};
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
const OTHERS_IN_THREE: usize = 2;
const CONTENT_DEADLINE: Duration = Duration::from_secs(60);
const QUIET: Duration = Duration::from_secs(3);
const CONVERGE_DEADLINE: Duration = Duration::from_secs(60);

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn brp_starboard_e2e() -> Result<(), String> {
    let net = BrpNetwork::prepare().await?;
    let bin = build_package_bin("e2e_apps", "starboard_e2e", "dev")?;
    let room = format!("brp-{}", unix_secs());
    let painter = StarboardPeer::launch(&bin, "painter", &net, PEER_WINDOWS[0]).await?;
    let helper = StarboardPeer::launch(&bin, "helper", &net, PEER_WINDOWS[1]).await?;
    let watcher = StarboardPeer::launch(&bin, "watcher", &net, PEER_WINDOWS[2]).await?;
    painter.create_room(&room).await?;
    helper.join_room(&room).await?;
    watcher.join_room(&room).await?;
    painter
        .wait_until_room_has_at_least_other_members(OTHERS_IN_THREE)
        .await?;
    helper
        .wait_until_room_has_at_least_other_members(OTHERS_IN_THREE)
        .await?;
    watcher
        .wait_until_room_has_at_least_other_members(OTHERS_IN_THREE)
        .await?;
    painter.paint_all(STROKES).await?;
    painter.wait_until_canvas_stops_changing(QUIET).await?;
    helper
        .wait_until_canvas_has_content(net.deadlines().scaled(CONTENT_DEADLINE))
        .await?;
    helper.erase_all(ERASE_STROKES).await?;
    helper.wait_until_canvas_stops_changing(QUIET).await?;
    watcher
        .wait_until_canvas_has_content(net.deadlines().scaled(CONTENT_DEADLINE))
        .await?;
    watcher.wait_until_canvas_stops_changing(QUIET).await?;
    expect_canvases_converge(
        &[&painter, &helper, &watcher],
        QUIET,
        net.deadlines().scaled(CONVERGE_DEADLINE),
    )
    .await?;
    painter.exit().await?;
    helper.exit().await?;
    watcher.exit().await
}
