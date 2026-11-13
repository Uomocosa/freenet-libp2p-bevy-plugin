use e2e_macros::e2e_test;
use freenet_libp2p_bevy_plugin::direct_dial::KnownPeerKind;
use freenet_libp2p_bevy_plugin::testing::FREENET_NODE_STARTED;

use crate::starboard_steps::{
    dial, exit_app, wait_until_friend_is_connected, wait_until_friend_is_disconnected,
};

// Known peers over real libp2p with no Freenet node: alice reaches bob with a
// one-off dial (no friend entry), so both books learn a PreviouslyConnected
// peer; alice stops while bob keeps running, then alice relaunches with the
// same identity and book and reaches the still-running bob with no add or
// dial step, still as a PreviouslyConnected peer.
#[e2e_test(game = "starboard", network = "direct-dial")]
fn known_peers(test: &mut Orchestrator) {
    test.peer("alice").runs([
        dial("bob"),
        wait_until_friend_is_connected("bob"),
        exit_app(),
    ]);
    test.peer("bob").runs([
        wait_until_friend_is_connected("alice"),
        wait_until_friend_is_disconnected("alice"),
        wait_until_friend_is_connected("alice"),
        exit_app(),
    ]);
    test.relaunch("alice")
        .runs([wait_until_friend_is_connected("bob"), exit_app()]);
    test.expect_book_kind("alice", "bob", KnownPeerKind::PreviouslyConnected);
    test.expect_book_kind("bob", "alice", KnownPeerKind::PreviouslyConnected);
    test.expect_no_log_line(FREENET_NODE_STARTED);
    test.expect_min_members(0);
}
