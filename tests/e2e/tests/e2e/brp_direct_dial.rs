use freenet_libp2p_bevy_plugin::testing::{FREENET_NODE_STARTED, Network};

use crate::brp::{BrpNetwork, DirectDialPeer, PEER_WINDOWS};
use crate::common::build_package_bin;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs an X display and the lavapipe ICD; run with --run-ignored all"]
async fn brp_direct_dial() -> Result<(), String> {
    let net = BrpNetwork::prepare_for(Network::DirectDial).await?;
    let bin = build_package_bin("e2e_apps", "starboard_e2e", "dev")?;
    let mut alice = DirectDialPeer::launch(&bin, "alice", &net, PEER_WINDOWS[0]).await?;
    let mut bob = DirectDialPeer::launch(&bin, "bob", &net, PEER_WINDOWS[1]).await?;
    let alice_status = alice.wait_until_listening().await?;
    let bob_status = bob.wait_until_listening().await?;
    alice.dial(&bob_status.listen_addrs.join(" ")).await?;
    alice.wait_until_connected_to(&bob_status.peer_id).await?;
    bob.wait_until_connected_to(&alice_status.peer_id).await?;
    alice.exit().await?;
    bob.wait_until_disconnected_from(&alice_status.peer_id)
        .await?;
    bob.exit().await?;
    if alice.log_contains(FREENET_NODE_STARTED) || bob.log_contains(FREENET_NODE_STARTED) {
        return Err(format!(
            "a peer started a freenet node: {FREENET_NODE_STARTED}"
        ));
    }
    Ok(())
}
