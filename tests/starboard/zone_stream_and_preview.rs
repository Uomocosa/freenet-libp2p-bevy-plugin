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
const STREAM_BUDGET: Duration = Duration::from_secs(60);
const PREVIEW_BUDGET: Duration = Duration::from_secs(60);
const LEASE_BUDGET: Duration = Duration::from_secs(60);
const CURSOR_BUDGET: Duration = Duration::from_secs(60);

async fn nudge(painter: &StarboardPeer, round: u32) -> Result<(), String> {
    painter
        .move_cursor_to_cell(if round.is_multiple_of(2) {
            (500.0, 500.0)
        } else {
            (520.0, 520.0)
        })
        .await
}

async fn paint_new_room(
    painter: &StarboardPeer,
    net: &BrpNetwork,
    room: &str,
) -> Result<(), String> {
    painter.create_room(room).await?;
    painter.wait_until_in_room(net.deadlines.action()).await?;
    painter.paint_all(STROKES).await?;
    painter.wait_until_canvas_stops_changing(QUIET).await?;
    wait_until_any_history_written(&[painter], net.deadlines.scaled(HISTORY_BUDGET)).await
}

async fn wait_until_stream_with_cursor(
    watcher: &StarboardPeer,
    painter_a: &StarboardPeer,
    painter_b: &StarboardPeer,
    room: &str,
    budget: Duration,
) -> Result<(), String> {
    let started = std::time::Instant::now();
    let mut round = 0_u32;
    loop {
        round = round.wrapping_add(1);
        nudge(painter_a, round).await?;
        nudge(painter_b, round).await?;
        let stream = watcher.stream_status().await?;
        if stream.is_showing_room(room) && stream.remote_cursors >= 1 {
            return Ok(());
        }
        if started.elapsed() > budget {
            return Err(format!(
                "watcher: no live stream of {room} with a cursor after {}s: {stream:?}\n--- watcher ---\n{}",
                started.elapsed().as_secs(),
                watcher.describe().await
            ));
        }
    }
}

async fn wait_until_preview_beside_stream(
    watcher: &StarboardPeer,
    painter_a: &StarboardPeer,
    painter_b: &StarboardPeer,
    previewed_room: &str,
    streamed_room: &str,
    budget: Duration,
) -> Result<(), String> {
    let started = std::time::Instant::now();
    let mut round = 0_u32;
    loop {
        round = round.wrapping_add(1);
        nudge(painter_a, round).await?;
        nudge(painter_b, round).await?;
        let preview = watcher.preview_status().await?;
        let stream = watcher.stream_status().await?;
        if preview.is_showing_room(previewed_room)
            && preview.remote_cursors >= 1
            && stream.is_showing_room(streamed_room)
            && stream.remote_cursors >= 1
        {
            return Ok(());
        }
        if started.elapsed() > budget {
            return Err(format!(
                "watcher: no live preview of {previewed_room} beside the stream of {streamed_room} after {}s: preview {preview:?}, stream {stream:?}\n--- watcher ---\n{}",
                started.elapsed().as_secs(),
                watcher.describe().await
            ));
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn starboard_zone_stream_and_preview() -> Result<(), String> {
    let net = BrpNetwork::prepare().await?;
    let mut recording = net.recording("starboard_zone_stream_and_preview");
    let bin = build_package_bin("starboard_e2e", "starboard_e2e", "dev")?;
    let room_a = format!("brp-a-{}", UnixTime::now().as_secs());
    let room_b = format!("brp-b-{}", UnixTime::now().as_secs());
    let painter_a = StarboardPeer::launch(&bin, "painter-a", &net, PEER_WINDOWS[0]).await?;
    let painter_b = StarboardPeer::launch(&bin, "painter-b", &net, PEER_WINDOWS[1]).await?;
    paint_new_room(&painter_a, &net, &room_a).await?;
    paint_new_room(&painter_b, &net, &room_b).await?;

    let watcher = StarboardPeer::launch(&bin, "watcher", &net, PEER_WINDOWS[2]).await?;
    watcher.wait_until_room_listed(&room_a).await?;
    watcher.wait_until_room_listed(&room_b).await?;

    watcher.click_room(&room_a).await?;
    wait_until_stream_with_cursor(
        &watcher,
        &painter_a,
        &painter_b,
        &room_a,
        net.deadlines.scaled(CURSOR_BUDGET),
    )
    .await?;
    let streaming_a = painter_a
        .wait_until_spectators(1, net.deadlines.scaled(LEASE_BUDGET))
        .await?;
    if streaming_a.spectators != 1 {
        return Err(format!(
            "painter-a: {} spectators while streamed, not 1",
            streaming_a.spectators
        ));
    }

    watcher.hover_room(&room_b).await?;
    wait_until_preview_beside_stream(
        &watcher,
        &painter_a,
        &painter_b,
        &room_b,
        &room_a,
        net.deadlines.scaled(CURSOR_BUDGET),
    )
    .await?;
    let previewing_b = painter_b
        .wait_until_spectators(1, net.deadlines.scaled(LEASE_BUDGET))
        .await?;
    if previewing_b.spectators != 1 {
        return Err(format!(
            "painter-b: {} spectators while previewed, not 1",
            previewing_b.spectators
        ));
    }
    let still_a = painter_a.canvas_status().await?;
    if still_a.spectators != 1 {
        return Err(format!(
            "painter-a: {} spectators while b is previewed, not 1",
            still_a.spectators
        ));
    }

    watcher.unhover().await?;
    watcher
        .wait_until_preview_cleared(net.deadlines.scaled(PREVIEW_BUDGET))
        .await?;
    let left_b = painter_b
        .wait_until_spectators_leave(net.deadlines.scaled(LEASE_BUDGET))
        .await?;
    if left_b.spectators != 0 {
        return Err(format!(
            "painter-b: {} spectators after the preview lease ended",
            left_b.spectators
        ));
    }
    let still_a = painter_a.canvas_status().await?;
    if still_a.spectators != 1 {
        return Err(format!(
            "painter-a: {} spectators after b's preview ended, not 1",
            still_a.spectators
        ));
    }

    watcher.close_stream().await?;
    watcher
        .wait_until_stream_cleared(net.deadlines.scaled(STREAM_BUDGET))
        .await?;
    let left_a = painter_a
        .wait_until_spectators_leave(net.deadlines.scaled(LEASE_BUDGET))
        .await?;
    if left_a.spectators != 0 {
        return Err(format!(
            "painter-a: {} spectators after the stream closed",
            left_a.spectators
        ));
    }

    painter_a.exit().await?;
    painter_b.exit().await?;
    watcher.exit().await?;
    recording.mark_passed();
    Ok(())
}
