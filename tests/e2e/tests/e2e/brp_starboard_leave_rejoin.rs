use std::time::Duration;

use crate::brp::{BrpNetwork, PEER_WINDOWS, StarboardPeer, Stroke, expect_canvases_converge};
use crate::common::build_package_bin;
use std::time::{SystemTime, UNIX_EPOCH};

const STROKES: &[Stroke] = &[
    Stroke(&[(200.0, 300.0), (260.0, 300.0)]),
    Stroke(&[(230.0, 309.0), (290.0, 309.0)]),
    Stroke(&[(260.0, 318.0), (320.0, 318.0)]),
    Stroke(&[(290.0, 327.0), (350.0, 327.0)]),
    Stroke(&[(320.0, 336.0), (380.0, 336.0)]),
    Stroke(&[(350.0, 345.0), (410.0, 345.0)]),
];
const BYSTANDER_MARK: Stroke = Stroke(&[(620.0, 640.0), (620.0, 640.0)]);
const PAINTER_CELL: (u16, u16) = (200, 300);
const BYSTANDER_CELL: (u16, u16) = (620, 640);
const OTHERS_IN_THREE: usize = 2;
const OTHERS_FEWER_THAN_THREE: usize = 1;
const QUIET: Duration = Duration::from_secs(3);
const CONVERGE_DEADLINE: Duration = Duration::from_secs(120);

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn brp_starboard_leave_rejoin() -> Result<(), String> {
    let net = BrpNetwork::prepare().await?;
    let bin = build_package_bin("e2e_apps", "starboard_e2e", "dev")?;
    let room = format!("brp-{}", unix_secs());
    let painter = StarboardPeer::launch(&bin, "painter", &net, PEER_WINDOWS[0]).await?;
    painter.create_room(&room).await?;
    let leaver = StarboardPeer::launch(&bin, "leaver", &net, PEER_WINDOWS[1]).await?;
    leaver.join_room(&room).await?;
    let bystander = StarboardPeer::launch(&bin, "bystander", &net, PEER_WINDOWS[2]).await?;
    bystander.join_room(&room).await?;
    painter
        .wait_until_room_has_other_members(OTHERS_IN_THREE)
        .await?;
    leaver
        .wait_until_room_has_other_members(OTHERS_IN_THREE)
        .await?;
    bystander
        .wait_until_room_has_other_members(OTHERS_IN_THREE)
        .await?;
    painter.paint_all(STROKES).await?;
    leaver.wait_until_cell_is_painted(PAINTER_CELL).await?;
    bystander.paint_all(&[BYSTANDER_MARK]).await?;
    leaver.wait_until_cell_is_painted(BYSTANDER_CELL).await?;
    leaver.leave_room().await?;
    painter
        .wait_until_room_has_at_most_other_members(OTHERS_FEWER_THAN_THREE)
        .await?;
    bystander
        .wait_until_room_has_at_most_other_members(OTHERS_FEWER_THAN_THREE)
        .await?;
    painter.paint_all(STROKES).await?;
    painter.wait_until_canvas_stops_changing(QUIET).await?;
    leaver.join_room(&room).await?;
    painter
        .wait_until_room_has_other_members(OTHERS_IN_THREE)
        .await?;
    leaver
        .wait_until_room_has_other_members(OTHERS_IN_THREE)
        .await?;
    bystander
        .wait_until_room_has_other_members(OTHERS_IN_THREE)
        .await?;
    painter.wait_until_canvas_stops_changing(QUIET).await?;
    leaver.wait_until_canvas_stops_changing(QUIET).await?;
    bystander.wait_until_canvas_stops_changing(QUIET).await?;
    expect_canvases_converge(
        &[&painter, &leaver, &bystander],
        QUIET,
        net.deadlines().scaled(CONVERGE_DEADLINE),
    )
    .await?;
    painter.exit().await?;
    leaver.exit().await?;
    bystander.exit().await
}
