use e2e_macros::e2e_test;

use crate::starboard_steps::{
    Cell, Stroke, create_room, exit_app, join_room, leave_room, paint, sleep_for_secs,
    wait_until_canvas_stops_changing, wait_until_cell_is_painted, wait_until_left_room,
    wait_until_room_has_fewer_members_than, wait_until_room_has_members,
};

// S3: p2 leaves, p1 paints over its cells, p2 rejoins and converges to the new hash.
// Every other peer is a bystander that sees the leave and the rejoin: it paints a mark
// before it watches for the leave, and p2 leaves only after it sees every mark.
const STROKES: &[Stroke] = &[
    Stroke::new(Cell::new(200, 300), Cell::new(260, 300)),
    Stroke::new(Cell::new(230, 309), Cell::new(290, 309)),
    Stroke::new(Cell::new(260, 318), Cell::new(320, 318)),
    Stroke::new(Cell::new(290, 327), Cell::new(350, 327)),
    Stroke::new(Cell::new(320, 336), Cell::new(380, 336)),
    Stroke::new(Cell::new(350, 345), Cell::new(410, 345)),
];

const BYSTANDER_MARK: Cell = Cell::new(620, 640);
const PAINTER_CELL: Cell = Cell::new(200, 300);

#[e2e_test(game = "starboard")]
fn starboard_leave_rejoin(test: &mut Orchestrator) {
    test.peer("painter").runs([
        create_room(),
        wait_until_room_has_members(3),
        paint(STROKES),
        wait_until_room_has_fewer_members_than(3),
        paint(STROKES),
        wait_until_room_has_members(3),
        wait_until_canvas_stops_changing(),
        exit_app(),
    ]);
    test.peer("leaver").runs([
        join_room(),
        wait_until_room_has_members(3),
        wait_until_cell_is_painted(PAINTER_CELL),
        wait_until_cell_is_painted(BYSTANDER_MARK),
        leave_room(),
        wait_until_left_room(),
        sleep_for_secs(20),
        join_room(),
        wait_until_room_has_members(3),
        wait_until_canvas_stops_changing(),
        exit_app(),
    ]);
    test.peer("bystander").runs([
        join_room(),
        wait_until_room_has_members(3),
        paint(&[Stroke::new(BYSTANDER_MARK, BYSTANDER_MARK)]),
        wait_until_room_has_fewer_members_than(3),
        wait_until_room_has_members(3),
        wait_until_canvas_stops_changing(),
        exit_app(),
    ]);
    test.expect_canvases_converge();
}
