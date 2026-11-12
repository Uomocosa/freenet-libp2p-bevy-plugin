use e2e_macros::e2e_test;
use freenet_libp2p_bevy_plugin::direct_dial::KnownPeerKind;
use freenet_libp2p_bevy_plugin::testing::ScriptCheckpoint;

use crate::starboard_steps::{
    add_friend, exit_app, mark_completed, sleep_for_secs, wait_until_friend_is_connected,
    wait_until_friend_is_disconnected,
};

// Bob's relaunch dials alice as soon as both gates hold, so it must never
// start before alice has logged his first exit: otherwise a slow-scripted
// alice (one script tick per frame on software rendering) can still be
// between her connect and disconnect waits when the reconnect lands, miss
// the offline window entirely and deadlock against the relaunched bob (both
// wait for the other to leave first).
const BOB_LEFT_ONCE: ScriptCheckpoint = ScriptCheckpoint::new("bob-left-once");

// Friends over real libp2p with no Freenet node: alice adds bob, both
// connect; bob restarts with the same identity and book while alice keeps
// running, so he dials her from his saved book and she reconnects without
// re-adding; then alice restarts too while the relaunched bob keeps running,
// so she dials him from her saved book, again with no add step.
#[e2e_test(game = "starboard", network = "direct-dial")]
fn friends_reconnect(test: &mut Orchestrator) {
    test.peer("alice").runs([
        add_friend("bob"),
        wait_until_friend_is_connected("bob"),
        wait_until_friend_is_disconnected("bob"),
        mark_completed(BOB_LEFT_ONCE),
        wait_until_friend_is_connected("bob"),
        exit_app(),
    ]);
    // Bob's first run must outlive alice's whole add phase: her eight
    // add-friend steps tick once per rendered frame, and on a slow frame
    // rate that outlasts the couple of seconds a plain connect-and-exit run
    // would live, leaving her connect wait nothing to observe.
    test.peer("bob").runs([
        wait_until_friend_is_connected("alice"),
        sleep_for_secs(45),
        exit_app(),
    ]);
    test.relaunch("bob").starts_after(BOB_LEFT_ONCE).runs([
        wait_until_friend_is_connected("alice"),
        wait_until_friend_is_disconnected("alice"),
        wait_until_friend_is_connected("alice"),
        exit_app(),
    ]);
    test.relaunch("alice")
        .runs([wait_until_friend_is_connected("bob"), exit_app()]);
    test.expect_book_kind("alice", "bob", KnownPeerKind::Friend);
    test.expect_book_kind("bob", "alice", KnownPeerKind::PreviouslyConnected);
    test.expect_no_freenet_node();
    test.expect_min_members(0);
}
