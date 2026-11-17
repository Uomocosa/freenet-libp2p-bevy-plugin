use freenet_libp2p_bevy_plugin::time::UnixTime;
use std::time::Duration;

use crate::brp::{BrpNetwork, PEER_WINDOWS, StarboardPeer, Stroke, wait_until_any_history_written};
use crate::common::build_package_bin;

const STROKES: &[Stroke] = &[
    Stroke(&[(200.0, 300.0), (260.0, 300.0)]),
    Stroke(&[(230.0, 309.0), (290.0, 309.0)]),
    Stroke(&[(260.0, 318.0), (320.0, 318.0)]),
];
const QUIET: Duration = Duration::from_secs(3);
const HISTORY_BUDGET: Duration = Duration::from_secs(120);
const PREVIEW_BUDGET: Duration = Duration::from_secs(60);
const LEASE_BUDGET: Duration = Duration::from_secs(60);
const CURSOR_BUDGET: Duration = Duration::from_secs(60);

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn starboard_room_preview() -> Result<(), String> {
    let net = BrpNetwork::prepare().await?;
    let mut recording = net.recording("starboard_room_preview");
    let bin = build_package_bin("starboard_e2e", "starboard_e2e", "dev")?;
    let room = format!("brp-{}", UnixTime::now().as_secs());
    let painter = StarboardPeer::launch(&bin, "painter", &net, PEER_WINDOWS[0]).await?;
    painter.create_room(&room).await?;
    painter.wait_until_in_room(net.deadlines.action()).await?;
    painter.paint_all(STROKES).await?;
    painter.wait_until_canvas_stops_changing(QUIET).await?;
    wait_until_any_history_written(&[&painter], net.deadlines.scaled(HISTORY_BUDGET)).await?;

    let watcher = StarboardPeer::launch(&bin, "watcher", &net, PEER_WINDOWS[1]).await?;
    watcher.wait_until_room_listed(&room).await?;
    watcher.hover_room(&room).await?;
    let preview = watcher
        .wait_until_preview_shows_room(&room, net.deadlines.scaled(PREVIEW_BUDGET))
        .await?;
    if preview.selected_room.as_deref() != Some(room.as_str()) {
        return Err(format!(
            "watcher: preview selected {:?}, not {room}",
            preview.selected_room
        ));
    }
    let observed = painter
        .wait_until_spectators(1, net.deadlines.scaled(LEASE_BUDGET))
        .await?;
    if observed.spectators != 1 {
        return Err(format!(
            "painter: {} spectators, not 1",
            observed.spectators
        ));
    }

    let started = std::time::Instant::now();
    let mut nudge = 0_u32;
    loop {
        nudge = nudge.wrapping_add(1);
        painter
            .move_cursor_to_cell(if nudge.is_multiple_of(2) {
                (500.0, 500.0)
            } else {
                (520.0, 520.0)
            })
            .await?;
        let status = watcher.preview_status().await?;
        if status.remote_cursors >= 1 {
            break;
        }
        if started.elapsed() > net.deadlines.scaled(CURSOR_BUDGET) {
            let painter_status = painter.canvas_status().await?;
            return Err(format!(
                "watcher: no preview cursor after {}s; painter own_cursor {:?}\n--- painter ---\n{}",
                started.elapsed().as_secs(),
                painter_status.own_cursor,
                painter.describe().await
            ));
        }
    }

    watcher.unhover().await?;
    watcher
        .wait_until_preview_cleared(net.deadlines.scaled(PREVIEW_BUDGET))
        .await?;
    let left = painter
        .wait_until_spectators_leave(net.deadlines.scaled(LEASE_BUDGET))
        .await?;
    if left.spectators != 0 {
        return Err(format!(
            "painter: {} spectators after the lease ended",
            left.spectators
        ));
    }

    painter.exit().await?;
    watcher.exit().await?;
    recording.mark_passed();
    Ok(())
}
