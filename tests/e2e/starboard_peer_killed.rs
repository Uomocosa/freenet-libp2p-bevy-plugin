use e2e_macros::e2e_test;

use crate::starboard_steps::{
    Cell, Stroke, crash_app, create_room, erase, exit_app, join_room, paint, sleep_for_secs,
    wait_until_canvas_has_content, wait_until_canvas_stops_changing,
    wait_until_room_has_at_most_members, wait_until_room_has_members,
};

// S6: the last peer crashes; the survivors drop it and keep converging.
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

#[e2e_test(game = "starboard")]
fn starboard_peer_killed(test: &mut Orchestrator) {
    test.peer("painter").runs([
        create_room(),
        paint(STROKES),
        wait_until_room_has_members(3),
        wait_until_canvas_has_content(),
        wait_until_room_has_at_most_members(2),
        erase(ERASE_STROKES),
        wait_until_canvas_stops_changing(),
        exit_app(),
    ]);
    test.peer("helper").runs([
        join_room(),
        wait_until_room_has_members(3),
        wait_until_canvas_has_content(),
        wait_until_room_has_at_most_members(2),
        wait_until_canvas_stops_changing(),
        exit_app(),
    ]);
    test.peer("crasher").runs([
        join_room(),
        wait_until_room_has_members(3),
        wait_until_canvas_has_content(),
        sleep_for_secs(5),
        crash_app(),
    ]);
    test.expect_canvases_converge();
    test.expect_peer_may_crash("crasher");
}
