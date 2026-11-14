use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::brp::{BrpNetwork, PEER_WINDOWS, StarboardPeer, Stroke};
use crate::common::build_package_bin;

const STROKES: &[Stroke] = &[
    Stroke(&[(200.0, 300.0), (260.0, 300.0)]),
    Stroke(&[(230.0, 309.0), (290.0, 309.0)]),
    Stroke(&[(260.0, 318.0), (320.0, 318.0)]),
    Stroke(&[(290.0, 327.0), (350.0, 327.0)]),
    Stroke(&[(320.0, 336.0), (380.0, 336.0)]),
    Stroke(&[(350.0, 345.0), (410.0, 345.0)]),
];
const FIRST_CELL: (u16, u16) = (200, 300);
const QUIET: Duration = Duration::from_secs(3);
const HOLDER_SETTLE: Duration = Duration::from_secs(5);

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn brp_starboard_history_holder_killed() -> Result<(), String> {
    let net = BrpNetwork::prepare().await?;
    let bin = build_package_bin("e2e_apps", "starboard_e2e", "dev")?;
    let room = format!("brp-{}", unix_secs());
    let mut holder = StarboardPeer::launch(&bin, "holder", &net, PEER_WINDOWS[0]).await?;
    holder.create_room(&room).await?;
    holder.wait_until_in_room(net.deadlines().action()).await?;
    let watcher = StarboardPeer::launch(&bin, "watcher", &net, PEER_WINDOWS[1]).await?;
    watcher.wait_until_room_listed(&room).await?;
    holder.paint_all(STROKES).await?;
    holder.wait_until_cell_is_painted(FIRST_CELL).await?;
    holder.wait_until_canvas_stops_changing(QUIET).await?;
    tokio::time::sleep(HOLDER_SETTLE).await;
    holder.kill()?;
    let late_joiner = StarboardPeer::launch(&bin, "late_joiner", &net, PEER_WINDOWS[2]).await?;
    late_joiner.wait_until_room_listed(&room).await?;
    late_joiner.join_room_expecting_failure(&room).await?;
    let painted_tiles = late_joiner.canvas_status().await?.painted_tiles;
    if painted_tiles != 0 {
        return Err(format!(
            "late_joiner: painted {painted_tiles} tiles without the history it could not fetch"
        ));
    }
    late_joiner.exit().await?;
    watcher.exit().await
}
