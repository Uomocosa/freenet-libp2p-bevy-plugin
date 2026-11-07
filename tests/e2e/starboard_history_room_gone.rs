use e2e_macros::e2e_test;
use freenet_libp2p_bevy_plugin::testing::ScriptCheckpoint;

use crate::starboard_steps::{
    Cell, Stroke, create_room, erase, exit_app, join_room, join_room_named, mark_completed, paint,
    sleep_for_secs, stay, wait_until_canvas_has_content, wait_until_canvas_stops_changing,
    wait_until_join_fails, wait_until_lobby_has_room, wait_until_room_has_members,
    wait_until_room_is_gone,
};

// S4b: the holders quit the game, the watchers stay in the lobby, and a later joiner
// finds the room gone.

// S4b: the holders quit the game, the watchers stay in the lobby, and a later joiner
// finds the room gone.
const STROKES: &[Stroke] = &[
    Stroke::new(Cell::new(200, 300), Cell::new(260, 300)),
    Stroke::new(Cell::new(230, 309), Cell::new(290, 309)),
    Stroke::new(Cell::new(260, 318), Cell::new(320, 318)),
    Stroke::new(Cell::new(290, 327), Cell::new(350, 327)),
    Stroke::new(Cell::new(320, 336), Cell::new(380, 336)),
    Stroke::new(Cell::new(350, 345), Cell::new(410, 345)),
];

const ERASE_STROKES: &[Stroke] = &[
    Stroke::new(Cell::new(200, 300), Cell::new(260, 300)),
    Stroke::new(Cell::new(230, 309), Cell::new(290, 309)),
    Stroke::new(Cell::new(260, 318), Cell::new(320, 318)),
];

const ROOM_GONE: ScriptCheckpoint = ScriptCheckpoint::new("room-gone");

#[e2e_test(game = "starboard")]
fn starboard_history_room_gone(test: &mut Orchestrator) {
    test.peer("painter").runs([
        create_room(),
        wait_until_room_has_members(2),
        paint(STROKES),
        wait_until_canvas_stops_changing(),
        sleep_for_secs(45),
        exit_app(),
    ]);
    test.peer("helper").runs([
        join_room(),
        wait_until_room_has_members(2),
        wait_until_canvas_has_content(),
        erase(ERASE_STROKES),
        wait_until_canvas_stops_changing(),
        sleep_for_secs(45),
        exit_app(),
    ]);
    test.peer("watcher").runs([
        wait_until_lobby_has_room(),
        wait_until_room_is_gone(),
        mark_completed(ROOM_GONE),
        stay(),
    ]);
    test.peer("late_joiner").starts_after(ROOM_GONE).runs([
        join_room_named("{room}"),
        wait_until_join_fails(),
        exit_app(),
    ]);
    test.expect_min_members(1);
    test.expect_peer_may_crash("watcher");
    test.expect_join_must_fail("late_joiner");
}
