use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::brp::{BrpNetwork, expect_meshed_within_budget, meet_in_room};
use crate::common::build_package_bin;

const PEERS: usize = 3;
const MESSAGES_DEADLINE: Duration = Duration::from_secs(60);
const SETTLE: Duration = Duration::from_secs(5);

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn brp_room_messaging() -> Result<(), String> {
    let net = BrpNetwork::prepare().await?;
    let bin = build_package_bin("e2e_apps", "lobby_room_e2e", "dev")?;
    let room = format!("brp-msg-{}", unix_secs());
    let (members, last_press) = meet_in_room(&bin, &net, PEERS, &room).await?;
    let converged =
        expect_meshed_within_budget(&members, last_press, net.deadlines().lobby()).await?;
    let others = PEERS.saturating_sub(1);
    for member in &members {
        member
            .wait_until_heard_from(others, MESSAGES_DEADLINE)
            .await?;
    }
    tokio::time::sleep(SETTLE).await;
    eprintln!("brp_room_messaging {converged}");
    for member in members {
        member.exit().await?;
    }
    Ok(())
}
