use std::time::{Duration, SystemTime, UNIX_EPOCH};

use freenet_libp2p_bevy_plugin::lobby_room::testing::LobbyKit;
use freenet_libp2p_bevy_plugin::testing::FakeNetworkServer;

use crate::brp::{StarboardPeer, Stroke, expect_canvases_converge, poll_until};
use crate::build_game::build_package_bin;

const STROKE_ACROSS: Stroke = Stroke(&[
    (120.0, 200.0),
    (200.0, 204.0),
    (300.0, 200.0),
    (400.0, 196.0),
    (460.0, 200.0),
]);
const STROKE_DOWN: Stroke = Stroke(&[
    (260.0, 100.0),
    (262.0, 160.0),
    (264.0, 240.0),
    (262.0, 300.0),
]);
const ERASE_OVER_CROSSING: Stroke = Stroke(&[(240.0, 200.0), (280.0, 200.0)]);
const ALICE_WINDOW: (i32, i32) = (0, 0);
// Bob's canvas centre must sit outside Alice's window (960 px wide, centre at x=480).
const BOB_WINDOW: (i32, i32) = (481, 0);
const SETTLE: Duration = Duration::from_secs(1);
const ERASE_DEADLINE: Duration = Duration::from_secs(10);
const QUIET: Duration = Duration::from_secs(3);
const CONVERGE_DEADLINE: Duration = Duration::from_secs(60);

fn unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos())
}

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn brp_concurrent_strokes_converge() -> Result<(), String> {
    let orchestrator = FakeNetworkServer::start_with(Box::new(LobbyKit::default())).await?;
    let bin = build_package_bin("e2e_apps", "starboard_e2e", "dev")?;
    let token = format!("brp-{}", unix_nanos());
    let room = format!("brp-{}", unix_secs());
    let alice = StarboardPeer::launch(&bin, "alice", &token, *orchestrator, ALICE_WINDOW).await?;
    let bob = StarboardPeer::launch(&bin, "bob", &token, *orchestrator, BOB_WINDOW).await?;
    alice.create_room(&room).await?;
    bob.join_room(&room).await?;
    if let Err(error) = alice.wait_until_room_has_other_members(1).await {
        return Err(format!("{error}\n{}", bob.describe().await));
    }
    bob.wait_until_room_has_other_members(1).await?;
    tokio::try_join!(alice.paint(STROKE_ACROSS), bob.paint(STROKE_DOWN))?;
    let before_erase = bob.wait_until_canvas_stops_changing(SETTLE).await?;
    bob.erase(ERASE_OVER_CROSSING).await?;
    poll_until(
        "bob's canvas changes after the erase",
        ERASE_DEADLINE,
        async || bob.canvas_status().await,
        |status| status.hash != before_erase.hash,
    )
    .await?;
    expect_canvases_converge(&[&alice, &bob], QUIET, CONVERGE_DEADLINE).await?;
    alice.exit().await?;
    bob.exit().await
}
