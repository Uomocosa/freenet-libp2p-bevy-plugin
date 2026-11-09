use e2e_macros::e2e_test;

use crate::starboard_steps::{
    Cell, Stroke, create_room, erase, exit_app, join_room, paint, wait_until_canvas_has_content,
    wait_until_canvas_stops_changing, wait_until_room_has_members,
};

// S1: p1 paints six strokes, p2 erases the first three, p3 only watches;
// every canvas ends the same, matching Scenario::starboard(3).
const STROKES: &[Stroke] = &[
    Stroke::new(Cell::new(200, 300), Cell::new(260, 300)),
    Stroke::new(Cell::new(230, 309), Cell::new(290, 309)),
    Stroke::new(Cell::new(260, 318), Cell::new(320, 318)),
    Stroke::new(Cell::new(290, 327), Cell::new(350, 327)),
    Stroke::new(Cell::new(320, 336), Cell::new(380, 336)),
    Stroke::new(Cell::new(350, 345), Cell::new(410, 345)),
];

// The three strokes p2 erases: the first three paint strokes, same geometry
// as the old erase script (see tests/common/scenario.rs).
const ERASE_STROKES: &[Stroke] = &[
    Stroke::new(Cell::new(200, 300), Cell::new(260, 300)),
    Stroke::new(Cell::new(230, 309), Cell::new(290, 309)),
    Stroke::new(Cell::new(260, 318), Cell::new(320, 318)),
];

#[e2e_test(game = "starboard")]
fn starboard_e2e(test: &mut Orchestrator) {
    test.peer("painter").runs([
        create_room(),
        wait_until_room_has_members(3),
        paint(STROKES),
        wait_until_canvas_stops_changing(),
        exit_app(),
    ]);
    test.peer("helper").runs([
        join_room(),
        wait_until_room_has_members(3),
        wait_until_canvas_has_content(),
        erase(ERASE_STROKES),
        wait_until_canvas_stops_changing(),
        exit_app(),
    ]);
    test.peer("watcher").runs([
        join_room(),
        wait_until_room_has_members(3),
        wait_until_canvas_has_content(),
        wait_until_canvas_stops_changing(),
        exit_app(),
    ]);
    test.expect_canvases_converge();
}
