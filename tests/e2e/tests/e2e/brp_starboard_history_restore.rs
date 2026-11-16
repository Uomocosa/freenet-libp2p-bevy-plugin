use freenet_libp2p_bevy_plugin::time::UnixTime;
use std::time::Duration;

use crate::brp::{
    BrpNetwork, Network, PEER_WINDOWS, StarboardPeer, Stroke, wait_until_any_history_written,
};
use crate::common::build_package_bin;

const STROKES: &[Stroke] = &[
    Stroke(&[(200.0, 300.0), (260.0, 300.0)]),
    Stroke(&[(230.0, 309.0), (290.0, 309.0)]),
    Stroke(&[(260.0, 318.0), (320.0, 318.0)]),
    Stroke(&[(290.0, 327.0), (350.0, 327.0)]),
    Stroke(&[(320.0, 336.0), (380.0, 336.0)]),
    Stroke(&[(350.0, 345.0), (410.0, 345.0)]),
];
const QUIET: Duration = Duration::from_secs(3);
const CONTENT_BUDGET: Duration = Duration::from_secs(120);
const HISTORY_BUDGET: Duration = Duration::from_secs(120);

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display, the lavapipe ICD and a real network; run with --run-ignored all"]
async fn brp_starboard_history_restore() -> Result<(), String> {
    Network::from_env().require_real("brp_starboard_history_restore")?;
    let net = BrpNetwork::prepare().await?;
    let bin = build_package_bin("e2e_apps", "starboard_e2e", "dev")?;
    let room = format!("brp-{}", UnixTime::now().as_secs());
    let painter = StarboardPeer::launch(&bin, "holder1", &net, PEER_WINDOWS[0]).await?;
    painter.create_room(&room).await?;
    painter.wait_until_in_room(net.deadlines.action()).await?;
    let helper = StarboardPeer::launch(&bin, "holder2", &net, PEER_WINDOWS[1]).await?;
    helper.join_room(&room).await?;
    painter
        .wait_until_room_has_at_least_other_members(1)
        .await?;
    helper.wait_until_room_has_at_least_other_members(1).await?;
    painter.paint_all(STROKES).await?;
    painter.wait_until_canvas_stops_changing(QUIET).await?;
    helper
        .wait_until_canvas_has_content(net.deadlines.scaled(CONTENT_BUDGET))
        .await?;
    helper.wait_until_canvas_stops_changing(QUIET).await?;
    wait_until_any_history_written(&[&painter, &helper], net.deadlines.scaled(HISTORY_BUDGET))
        .await?;
    let painted_hash = painter.canvas_status().await?.hash;
    painter.leave_room().await?;
    helper.wait_until_room_has_at_most_other_members(0).await?;
    helper.leave_room().await?;
    let late_joiner = StarboardPeer::launch(&bin, "late_joiner", &net, PEER_WINDOWS[2]).await?;
    late_joiner.join_room(&room).await?;
    late_joiner
        .wait_until_canvas_has_content(net.deadlines.scaled(CONTENT_BUDGET))
        .await?;
    let restored = late_joiner.wait_until_canvas_stops_changing(QUIET).await?;
    if restored.hash != painted_hash {
        return Err(format!(
            "late_joiner: restored hash {:?} differs from the painter's {painted_hash:?}",
            restored.hash
        ));
    }
    late_joiner.exit().await?;
    helper.exit().await?;
    painter.exit().await
}
