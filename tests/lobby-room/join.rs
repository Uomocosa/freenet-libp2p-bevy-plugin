use freenet_libp2p_bevy_plugin::time::UnixTime;
use std::time::Duration;

use crate::brp::{BrpNetwork, expect_meshed_within_budget, meet_in_room};
use crate::common::build_package_bin;

const PEERS: usize = 5;
const SETTLE: Duration = Duration::from_secs(5);

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn lobby_room_join() -> Result<(), String> {
    let net = BrpNetwork::prepare().await?;
    let mut recording = net.recording("lobby_room_join");
    let bin = build_package_bin("lobby_room_e2e", "lobby_room_e2e", "dev")?;
    let room = format!("brp-join-{}", UnixTime::now().as_secs());
    let (members, last_press) = meet_in_room(&bin, &net, PEERS, &room).await?;
    let converged =
        expect_meshed_within_budget(&members, last_press, net.deadlines.lobby()).await?;
    tokio::time::sleep(SETTLE).await;
    eprintln!("lobby_room_join {converged}");
    for member in members {
        member.exit().await?;
    }
    recording.mark_passed();
    Ok(())
}
