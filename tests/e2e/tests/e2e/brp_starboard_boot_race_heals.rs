use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::brp::{BrpNetwork, PEER_WINDOWS, StarboardPeer, require_real_network};
use crate::common::build_package_bin;

const SETTLE: Duration = Duration::from_secs(5);

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display, the lavapipe ICD and a real network; run with --run-ignored all"]
async fn brp_starboard_boot_race_heals() -> Result<(), String> {
    require_real_network("brp_starboard_boot_race_heals")?;
    let net = BrpNetwork::prepare_with(
        freenet_libp2p_bevy_plugin::testing::Network::from_env(),
        crate::common::LobbySetup::Absent,
    )
    .await?;
    let bin = build_package_bin("e2e_apps", "starboard_e2e", "dev")?;
    let room = format!("brp-{}", unix_secs());
    let (creator, joiner) = tokio::try_join!(
        StarboardPeer::launch(&bin, "p1", &net, PEER_WINDOWS[0]),
        StarboardPeer::launch(&bin, "p2", &net, PEER_WINDOWS[1]),
    )?;
    creator.create_room(&room).await?;
    joiner.wait_until_room_listed(&room).await?;
    joiner.join_room(&room).await?;
    creator
        .wait_until_room_has_at_least_other_members(1)
        .await?;
    joiner.wait_until_room_has_at_least_other_members(1).await?;
    tokio::time::sleep(SETTLE).await;
    creator.exit().await?;
    joiner.exit().await
}
