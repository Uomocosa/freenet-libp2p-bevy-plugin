use e2e_macros::e2e_test;
use freenet_libp2p_bevy_plugin::testing::Checkpoint;

use crate::starboard_steps::{
    Cell, Stroke, create_room, exit_app, join_room, mark_completed, paint,
    wait_until_canvas_has_content, wait_until_canvas_stops_changing, wait_until_room_has_members,
};

const STROKES: &[Stroke] = &[
    Stroke::new(Cell::new(200, 300), Cell::new(260, 300)),
    Stroke::new(Cell::new(230, 309), Cell::new(290, 309)),
];
const PAINTING_DONE: Checkpoint = Checkpoint::new("painting-done");

#[e2e_test(game = "starboard")]
fn starboard_late_join(test: &mut Orchestrator) {
    test.peer("painter").runs([
        create_room(),
        wait_until_room_has_members(2),
        paint(STROKES),
        mark_completed(PAINTING_DONE),
        wait_until_room_has_members(3),
        wait_until_canvas_stops_changing(),
        exit_app(),
    ]);
    test.peer("helper").runs([
        join_room(),
        wait_until_room_has_members(2),
        wait_until_canvas_has_content(),
        wait_until_room_has_members(3),
        wait_until_canvas_stops_changing(),
        exit_app(),
    ]);
    test.peer("late_joiner").starts_after(PAINTING_DONE).runs([
        join_room(),
        wait_until_canvas_has_content(),
        wait_until_canvas_stops_changing(),
        exit_app(),
    ]);
    test.expect_canvases_converge();
}
