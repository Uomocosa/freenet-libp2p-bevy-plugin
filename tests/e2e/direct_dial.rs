use e2e_macros::e2e_test;

use crate::starboard_steps::{
    dial, exit_app, wait_until_friend_is_connected, wait_until_friend_is_disconnected,
};

// Direct dial over real libp2p with no Freenet node: alice dials bob's live
// address once (no friend entry), both see each other connected.
#[e2e_test(game = "starboard", network = "direct-dial")]
fn direct_dial(test: &mut Orchestrator) {
    test.peer("alice").runs([
        dial("bob"),
        wait_until_friend_is_connected("bob"),
        exit_app(),
    ]);
    test.peer("bob").runs([
        wait_until_friend_is_connected("alice"),
        wait_until_friend_is_disconnected("alice"),
        exit_app(),
    ]);
    test.expect_no_freenet_node();
    test.expect_min_members(0);
}
