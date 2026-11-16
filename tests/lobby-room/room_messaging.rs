use freenet_libp2p_bevy_plugin::time::UnixTime;
use std::time::Duration;

use crate::brp::{BrpNetwork, expect_meshed_within_budget, meet_in_room};
use crate::common::build_package_bin;

const PEERS: usize = 3;
const MESSAGES_DEADLINE: Duration = Duration::from_secs(60);
const SETTLE: Duration = Duration::from_secs(5);

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn room_messaging() -> Result<(), String> {
    let net = BrpNetwork::prepare().await?;
    let bin = build_package_bin("lobby_room_e2e", "lobby_room_e2e", "dev")?;
    let room = format!("brp-msg-{}", UnixTime::now().as_secs());
    let (members, last_press) = meet_in_room(&bin, &net, PEERS, &room).await?;
    let converged =
        expect_meshed_within_budget(&members, last_press, net.deadlines.lobby()).await?;
    let others = PEERS.saturating_sub(1);
    for member in &members {
        member
            .wait_until_heard_from(others, MESSAGES_DEADLINE)
            .await?;
    }
    tokio::time::sleep(SETTLE).await;
    eprintln!("room_messaging {converged}");
    for member in members {
        member.exit().await?;
    }
    Ok(())
}
