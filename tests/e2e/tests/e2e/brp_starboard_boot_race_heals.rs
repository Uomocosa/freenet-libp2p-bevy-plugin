use freenet_libp2p_bevy_plugin::time::UnixTime;
use std::time::Duration;

use crate::brp::{BrpNetwork, LobbySetup, Network, PEER_WINDOWS, StarboardPeer};
use crate::common::build_package_bin;

const SETTLE: Duration = Duration::from_secs(5);

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display, the lavapipe ICD and a real network; run with --run-ignored all"]
async fn brp_starboard_boot_race_heals() -> Result<(), String> {
    Network::from_env().require_real("brp_starboard_boot_race_heals")?;
    let net = BrpNetwork::try_from_network_setup(
        freenet_libp2p_bevy_plugin::testing::Network::from_env(),
        LobbySetup::Absent,
    )
    .await?;
    let bin = build_package_bin("e2e_apps", "starboard_e2e", "dev")?;
    let room = format!("brp-{}", UnixTime::now().as_secs());
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
