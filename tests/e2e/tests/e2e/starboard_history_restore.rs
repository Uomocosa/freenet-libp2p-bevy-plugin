use e2e_macros::e2e_test;
use freenet_libp2p_bevy_plugin::testing::ScriptCheckpoint;

use crate::starboard_steps::{
    Cell, Stroke, create_room, exit_app, join_room, leave_room, mark_completed, paint,
    sleep_for_secs, stay, wait_until_canvas_has_content, wait_until_canvas_stops_changing,
    wait_until_left_room, wait_until_room_has_fewer_members_than, wait_until_room_has_members,
};

// S4a: the holders paint, stay while the history is written, and leave the room;
// a late joiner restores the board from the Kademlia history.
const STROKES: &[Stroke] = &[
    Stroke::new(Cell::new(200, 300), Cell::new(260, 300)),
    Stroke::new(Cell::new(230, 309), Cell::new(290, 309)),
    Stroke::new(Cell::new(260, 318), Cell::new(320, 318)),
    Stroke::new(Cell::new(290, 327), Cell::new(350, 327)),
    Stroke::new(Cell::new(320, 336), Cell::new(380, 336)),
    Stroke::new(Cell::new(350, 345), Cell::new(410, 345)),
];

const HOLDERS_LEFT: ScriptCheckpoint = ScriptCheckpoint::new("holders-left");

#[e2e_test(game = "starboard")]
fn starboard_history_restore(test: &mut Orchestrator) {
    test.peer("holder1").runs([
        create_room(),
        wait_until_room_has_members(2),
        paint(STROKES),
        wait_until_canvas_stops_changing(),
        sleep_for_secs(45),
        leave_room(),
        wait_until_left_room(),
        mark_completed(ScriptCheckpoint::new("holder1-left")),
        stay(),
    ]);
    test.peer("holder2").runs([
        join_room(),
        wait_until_room_has_members(2),
        wait_until_canvas_has_content(),
        sleep_for_secs(45),
        wait_until_room_has_fewer_members_than(2),
        leave_room(),
        wait_until_left_room(),
        mark_completed(HOLDERS_LEFT),
        stay(),
    ]);
    test.peer("late_joiner").starts_after(HOLDERS_LEFT).runs([
        join_room(),
        wait_until_canvas_has_content(),
        wait_until_canvas_stops_changing(),
        exit_app(),
    ]);
    test.expect_canvases_converge();
    test.expect_min_members(1);
    test.expect_history_written_and_restored();
}
