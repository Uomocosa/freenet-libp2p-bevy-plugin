use freenet_libp2p_bevy_plugin::direct_dial::KnownPeerKind;
use freenet_libp2p_bevy_plugin::testing::{FREENET_NODE_STARTED, Network};

use crate::brp::{BrpNetwork, DirectDialPeer, PEER_WINDOWS};
use crate::common::build_package_bin;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn brp_friends_reconnect() -> Result<(), String> {
    let net = BrpNetwork::prepare_for(Network::DirectDial).await?;
    let bin = build_package_bin("e2e_apps", "starboard_e2e", "dev")?;
    let mut alice = DirectDialPeer::launch(&bin, "alice", &net, PEER_WINDOWS[0]).await?;
    let mut bob = DirectDialPeer::launch(&bin, "bob", &net, PEER_WINDOWS[1]).await?;
    let alice_status = alice.wait_until_listening().await?;
    let bob_status = bob.wait_until_listening().await?;
    let bob_addr = bob_status
        .listen_addrs
        .first()
        .ok_or_else(|| "bob reports no listen address".to_string())?;
    alice.add_friend_through_ui(bob_addr).await?;
    alice
        .wait_until_book_kind(&bob_status.peer_id, KnownPeerKind::Friend)
        .await?;
    alice.wait_until_connected_to(&bob_status.peer_id).await?;
    bob.wait_until_connected_to(&alice_status.peer_id).await?;
    bob.wait_until_book_has_addrs_of(&alice_status.peer_id)
        .await?;
    bob.exit().await?;
    alice
        .wait_until_disconnected_from(&bob_status.peer_id)
        .await?;
    bob.relaunch().await?;
    let bob_relaunched = bob.wait_until_listening().await?;
    if bob_relaunched.peer_id != bob_status.peer_id {
        return Err(format!(
            "bob relaunched as {} instead of {}",
            bob_relaunched.peer_id, bob_status.peer_id
        ));
    }
    bob.wait_until_connected_to(&alice_status.peer_id).await?;
    alice.wait_until_connected_to(&bob_status.peer_id).await?;
    alice
        .wait_until_book_has_addrs_of(&bob_status.peer_id)
        .await?;
    alice.exit().await?;
    bob.wait_until_disconnected_from(&alice_status.peer_id)
        .await?;
    alice.relaunch().await?;
    let alice_relaunched = alice.wait_until_listening().await?;
    if alice_relaunched.peer_id != alice_status.peer_id {
        return Err(format!(
            "alice relaunched as {} instead of {}",
            alice_relaunched.peer_id, alice_status.peer_id
        ));
    }
    alice.wait_until_connected_to(&bob_status.peer_id).await?;
    bob.wait_until_connected_to(&alice_status.peer_id).await?;
    let alice_kind = alice.book_kind(&bob_status.peer_id).await?;
    if alice_kind != Some(KnownPeerKind::Friend) {
        return Err(format!("alice's book has bob as {alice_kind:?}"));
    }
    let bob_kind = bob.book_kind(&alice_status.peer_id).await?;
    if bob_kind != Some(KnownPeerKind::PreviouslyConnected) {
        return Err(format!("bob's book has alice as {bob_kind:?}"));
    }
    alice.exit().await?;
    bob.exit().await?;
    if alice.log_contains(FREENET_NODE_STARTED) || bob.log_contains(FREENET_NODE_STARTED) {
        return Err(format!(
            "a peer started a freenet node: {FREENET_NODE_STARTED}"
        ));
    }
    Ok(())
}
