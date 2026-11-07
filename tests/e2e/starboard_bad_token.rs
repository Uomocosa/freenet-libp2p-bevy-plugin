use e2e_macros::e2e_test;
use freenet_libp2p_bevy_plugin::testing::Checkpoint;

use crate::starboard_steps::{
    create_room, exit_app, join_room_named, mark_completed, sleep_for_secs, stay,
    wait_until_join_fails, wait_until_join_fails_named, wait_until_lobby_has_room,
};

// S7: a bad token and a missing room both fail to join.
const CREATOR_READY: Checkpoint = Checkpoint::new("creator-ready");

#[e2e_test(game = "starboard")]
fn starboard_bad_token(test: &mut Orchestrator) {
    test.peer("creator").runs([
        create_room(),
        mark_completed(CREATOR_READY),
        sleep_for_secs(240),
        exit_app(),
    ]);
    test.peer("bad_token")
        .with_token("lobby-e2e/bad-token")
        .starts_after(CREATOR_READY)
        .runs([
            join_room_named("{room}"),
            wait_until_join_fails(),
            exit_app(),
        ]);
    test.peer("missing_room").starts_after(CREATOR_READY).runs([
        join_room_named("no-such-room-{room}"),
        wait_until_join_fails_named("no-such-room-{room}"),
        exit_app(),
    ]);
    test.peer("watcher")
        .runs([wait_until_lobby_has_room(), stay()]);
    test.expect_min_members(0);
    test.expect_peer_may_crash("watcher");
    test.expect_join_must_fail("bad_token");
    test.expect_join_must_fail("missing_room");
}
