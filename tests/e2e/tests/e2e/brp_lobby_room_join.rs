use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::brp::{BrpNetwork, expect_meshed_within_budget, meet_in_room};
use crate::common::build_package_bin;

const PEERS: usize = 5;
const SETTLE: Duration = Duration::from_secs(5);

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn brp_lobby_room_join() -> Result<(), String> {
    let net = BrpNetwork::prepare().await?;
    let bin = build_package_bin("e2e_apps", "lobby_room_e2e", "dev")?;
    let room = format!("brp-join-{}", unix_secs());
    let (members, last_press) = meet_in_room(&bin, &net, PEERS, &room).await?;
    let converged =
        expect_meshed_within_budget(&members, last_press, net.deadlines().lobby()).await?;
    tokio::time::sleep(SETTLE).await;
    eprintln!("brp_lobby_room_join {converged}");
    for member in members {
        member.exit().await?;
    }
    Ok(())
}
