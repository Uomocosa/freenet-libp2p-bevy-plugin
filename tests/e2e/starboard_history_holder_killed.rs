use e2e_macros::e2e_test;
use freenet_libp2p_bevy_plugin::testing::ScriptCheckpoint;

use crate::starboard_steps::{
    Cell, Stroke, crash_app, create_room, exit_app, join_room_named, mark_completed, paint,
    sleep_for_secs, stay, wait_until_canvas_stops_changing, wait_until_join_fails,
    wait_until_lobby_has_room,
};

// S4c: the only holder crashes as the joiner starts; the joiner must report the room gone
// once the dead holder's lease lapses, never accept an empty board.
const STROKES: &[Stroke] = &[
    Stroke::new(Cell::new(200, 300), Cell::new(260, 300)),
    Stroke::new(Cell::new(230, 309), Cell::new(290, 309)),
    Stroke::new(Cell::new(260, 318), Cell::new(320, 318)),
    Stroke::new(Cell::new(290, 327), Cell::new(350, 327)),
    Stroke::new(Cell::new(320, 336), Cell::new(380, 336)),
    Stroke::new(Cell::new(350, 345), Cell::new(410, 345)),
];

const HOLDER_DYING: ScriptCheckpoint = ScriptCheckpoint::new("holder-dying");

#[e2e_test(game = "starboard")]
fn starboard_history_holder_killed(test: &mut Orchestrator) {
    test.peer("holder").runs([
        create_room(),
        paint(STROKES),
        wait_until_canvas_stops_changing(),
        sleep_for_secs(5),
        mark_completed(HOLDER_DYING),
        crash_app(),
    ]);
    test.peer("watcher")
        .runs([wait_until_lobby_has_room(), stay()]);
    test.peer("late_joiner").starts_after(HOLDER_DYING).runs([
        wait_until_lobby_has_room(),
        join_room_named("{room}"),
        wait_until_join_fails(),
        exit_app(),
    ]);
    test.expect_min_members(1);
    test.expect_peer_may_crash("holder");
    test.expect_peer_may_crash("watcher");
    test.expect_join_must_fail("late_joiner");
}
