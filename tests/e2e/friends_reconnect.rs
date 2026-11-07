use e2e_macros::e2e_test;
use freenet_libp2p_bevy_plugin::direct_dial::KnownPeerKind;

use crate::starboard_steps::{
    add_friend, exit_app, wait_until_friend_is_connected, wait_until_friend_is_disconnected,
};

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
        wait_until_friend_is_connected("bob"),
        exit_app(),
    ]);
    test.peer("bob")
        .runs([wait_until_friend_is_connected("alice"), exit_app()]);
    test.relaunch("bob").runs([
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
