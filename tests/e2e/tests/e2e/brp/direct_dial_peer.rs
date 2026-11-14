use std::path::Path;
use std::time::Duration;

use freenet_libp2p_bevy_plugin::testing::DirectDialStatus;
use serde_json::Value;

use super::{BrpNetwork, Deadlines, Peer, poll_until};

const LINK_BUDGET: Duration = Duration::from_secs(120);

/// One `starboard_e2e` process on the direct-dial network, with its earlier runs' logs kept
/// readable so the no-node check covers every launch of the peer.
pub struct DirectDialPeer {
    name: String,
    deadlines: Deadlines,
    peer: Peer,
}

impl DirectDialPeer {
    /// # Errors
    /// Returns an error if the app does not start or does not answer over BRP.
    pub async fn launch(
        bin: &Path,
        name: &str,
        net: &BrpNetwork,
        window: (i32, i32),
    ) -> Result<Self, String> {
        let args = net.peer_args(name, window);
        let peer = Peer::launch(bin, &args).await?;
        Ok(Self {
            name: name.to_string(),
            deadlines: net.deadlines(),
            peer,
        })
    }

    /// # Errors
    /// Returns an error if the request fails or the peer does not answer.
    pub async fn status(&self) -> Result<DirectDialStatus, String> {
        self.peer
            .call("direct_dial/status", Value::Null)
            .await
            .map_err(|error| self.peer.explain(&error))
    }

    /// Waits until the peer has its own peer id and at least one dialable listen address.
    ///
    /// # Errors
    /// Returns an error if the peer does not report a listen address in time.
    pub async fn wait_until_listening(&self) -> Result<DirectDialStatus, String> {
        poll_until(
            &format!("{}: listen addresses", self.name),
            self.deadlines.scaled(LINK_BUDGET),
            async || self.status().await,
            |status| !status.listen_addrs.is_empty() && !status.peer_id.is_empty(),
        )
        .await
    }

    /// # Errors
    /// Returns an error if the dial command cannot be sent.
    pub async fn dial(&self, addrs: &str) -> Result<(), String> {
        self.peer
            .call::<Value>("e2e/dial", Value::String(addrs.to_string()))
            .await
            .map(|_| ())
            .map_err(|error| self.peer.explain(&error))
    }

    /// # Errors
    /// Returns an error if `peer_id` is never connected.
    pub async fn wait_until_connected_to(&self, peer_id: &str) -> Result<(), String> {
        poll_until(
            &format!("{}: connected to {peer_id}", self.name),
            self.deadlines.scaled(LINK_BUDGET),
            async || self.status().await,
            |status| status.connected.iter().any(|id| id == peer_id),
        )
        .await
        .map(|_| ())
    }

    /// # Errors
    /// Returns an error if `peer_id` stays connected.
    pub async fn wait_until_disconnected_from(&self, peer_id: &str) -> Result<(), String> {
        poll_until(
            &format!("{}: disconnected from {peer_id}", self.name),
            self.deadlines.scaled(LINK_BUDGET),
            async || self.status().await,
            |status| !status.connected.iter().any(|id| id == peer_id),
        )
        .await
        .map(|_| ())
    }

    /// # Errors
    /// Returns an error if the app does not exit in time or exits unsuccessfully.
    pub async fn exit(&mut self) -> Result<(), String> {
        self.peer.exit().await
    }

    #[must_use]
    pub fn log_contains(&self, line: &str) -> bool {
        self.peer.log_contains(line)
    }
}
