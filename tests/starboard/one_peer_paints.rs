use freenet_libp2p_bevy_plugin::time::UnixTime;
use std::time::Duration;

use crate::brp::{BrpNetwork, StarboardPeer, Stroke};
use crate::common::build_package_bin;

const LINE: Stroke = Stroke(&[
    (100.0, 100.0),
    (120.0, 104.0),
    (140.0, 112.0),
    (160.0, 124.0),
    (180.0, 130.0),
]);
const CONTENT_DEADLINE: Duration = Duration::from_secs(60);
const QUIET: Duration = Duration::from_secs(2);

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn one_peer_paints() -> Result<(), String> {
    let net = BrpNetwork::prepare().await?;
    let mut recording = net.recording("one_peer_paints");
    let bin = build_package_bin("starboard_e2e", "starboard_e2e", "dev")?;
    let room = format!("brp-{}", UnixTime::now().as_secs());
    let alice = StarboardPeer::launch(&bin, "alice", &net, (0, 0)).await?;
    alice.create_room(&room).await?;
    alice.wait_until_in_room(net.deadlines.action()).await?;
    alice.paint(LINE).await?;
    alice
        .wait_until_canvas_has_content(net.deadlines.scaled(CONTENT_DEADLINE))
        .await?;
    let settled = alice.wait_until_canvas_stops_changing(QUIET).await?;
    if settled.painted_tiles == 0 {
        return Err(format!("canvas has no painted tile: {settled:?}"));
    }
    alice.exit().await?;
    recording.mark_passed();
    Ok(())
}
