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
const OTHERS_IN_TWO: usize = 1;
const CONTENT_DEADLINE: Duration = Duration::from_secs(60);
const CRASH_AFTER: Duration = Duration::from_secs(5);
const QUIET: Duration = Duration::from_secs(3);
const CONVERGE_DEADLINE: Duration = Duration::from_secs(60);

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn brp_starboard_peer_killed() -> Result<(), String> {
    let net = BrpNetwork::prepare().await?;
    let bin = build_package_bin("e2e_apps", "starboard_e2e", "dev")?;
    let room = format!("brp-{}", unix_secs());
    let alice = StarboardPeer::launch(&bin, "painter", &net, PEER_WINDOWS[0]).await?;
    let bob = StarboardPeer::launch(&bin, "helper", &net, PEER_WINDOWS[1]).await?;
    let mut carol = StarboardPeer::launch(&bin, "crasher", &net, PEER_WINDOWS[2]).await?;
    alice.create_room(&room).await?;
    alice.paint_all(STROKES).await?;
    bob.join_room(&room).await?;
    carol.join_room(&room).await?;
    alice
        .wait_until_room_has_at_least_other_members(OTHERS_IN_THREE)
        .await?;
    bob.wait_until_room_has_at_least_other_members(OTHERS_IN_THREE)
        .await?;
    carol
        .wait_until_room_has_at_least_other_members(OTHERS_IN_THREE)
        .await?;
    alice
        .wait_until_canvas_has_content(net.deadlines().scaled(CONTENT_DEADLINE))
        .await?;
    bob.wait_until_canvas_has_content(net.deadlines().scaled(CONTENT_DEADLINE))
        .await?;
    carol
        .wait_until_canvas_has_content(net.deadlines().scaled(CONTENT_DEADLINE))
        .await?;
    tokio::time::sleep(net.deadlines().scaled(CRASH_AFTER)).await;
    carol.kill()?;
    alice
        .wait_until_room_has_at_most_other_members(OTHERS_IN_TWO)
        .await?;
    bob.wait_until_room_has_at_most_other_members(OTHERS_IN_TWO)
        .await?;
    alice.erase_all(ERASE_STROKES).await?;
    alice.wait_until_canvas_stops_changing(QUIET).await?;
    bob.wait_until_canvas_stops_changing(QUIET).await?;
    expect_canvases_converge(
        &[&alice, &bob],
        QUIET,
        net.deadlines().scaled(CONVERGE_DEADLINE),
    )
    .await?;
    alice.exit().await?;
    bob.exit().await
}
