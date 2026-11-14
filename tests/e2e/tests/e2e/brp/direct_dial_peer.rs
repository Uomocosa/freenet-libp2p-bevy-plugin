use std::path::{Path, PathBuf};
use std::time::Duration;

use freenet_libp2p_bevy_plugin::direct_dial::KnownPeerKind;
use freenet_libp2p_bevy_plugin::lobby_room::testing::{LobbyTarget, LocatedNode};
use freenet_libp2p_bevy_plugin::testing::{CursorTarget, DirectDialStatus, NamedKey, PointerSpeed};
use serde_json::{Value, json};

use super::{BrpNetwork, Deadlines, Peer, poll_until};

const LINK_BUDGET: Duration = Duration::from_secs(120);
const CLICK_SPEED: f32 = 1200.0;

/// One `starboard_e2e` process on the direct-dial network, with its earlier runs' logs kept
/// readable so the no-node check covers every launch of the peer.
pub struct DirectDialPeer {
    name: String,
    bin: PathBuf,
    args: Vec<String>,
    deadlines: Deadlines,
    peer: Peer,
    earlier_logs: Vec<String>,
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
            bin: bin.to_path_buf(),
            args,
            deadlines: net.deadlines(),
            peer,
            earlier_logs: Vec::new(),
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

    /// The kind the peer book records for `peer_id`, if it has an entry.
    ///
    /// # Errors
    /// Returns an error if the status request fails.
    pub async fn book_kind(&self, peer_id: &str) -> Result<Option<KnownPeerKind>, String> {
        let status = self.status().await?;
        Ok(status
            .book
            .iter()
            .find(|entry| entry.peer_id == peer_id)
            .map(|entry| entry.kind))
    }

    /// Waits until the peer book file on disk holds `peer_id` with at least one address. The
    /// book entry appears when the connection is made, but its addresses only arrive with the
    /// identify exchange; a peer that exits before that relaunches knowing nobody it can dial.
    ///
    /// # Errors
    /// Returns an error if the book file never holds an address for `peer_id` in time.
    pub async fn wait_until_book_has_addrs_of(&self, peer_id: &str) -> Result<(), String> {
        let path = self
            .args
            .iter()
            .position(|arg| arg == "--peer-book-file")
            .and_then(|at| at.checked_add(1))
            .and_then(|next| self.args.get(next))
            .ok_or_else(|| format!("{}: no --peer-book-file argument", self.name))?
            .clone();
        poll_until(
            &format!("{}: persisted addresses of {peer_id}", self.name),
            self.deadlines.scaled(LINK_BUDGET),
            async || {
                let text = std::fs::read_to_string(&path).unwrap_or_default();
                Ok(serde_json::from_str::<Value>(&text).unwrap_or(Value::Null))
            },
            |book| {
                book.as_array().is_some_and(|entries| {
                    entries.iter().any(|entry| {
                        let peer = entry.get("peer");
                        peer.and_then(|peer| peer.get("id"))
                            .is_some_and(|id| *id == peer_id)
                            && peer
                                .and_then(|peer| peer.get("addrs"))
                                .and_then(Value::as_array)
                                .is_some_and(|addrs| !addrs.is_empty())
                    })
                })
            },
        )
        .await
        .map(|_| ())
    }

    /// Waits until the peer book holds `peer_id` with the given kind.
    ///
    /// # Errors
    /// Returns an error if the book never records `peer_id` as `kind` in time.
    pub async fn wait_until_book_kind(
        &self,
        peer_id: &str,
        kind: KnownPeerKind,
    ) -> Result<(), String> {
        poll_until(
            &format!("{}: {peer_id} booked as {kind:?}", self.name),
            self.deadlines.scaled(LINK_BUDGET),
            async || self.book_kind(peer_id).await,
            |found| *found == Some(kind),
        )
        .await
        .map(|_| ())
    }

    /// Adds `addr` as a friend the way a player does: Tab opens the friends drawer, the address
    /// field is clicked and typed into, and the add button is clicked once it is enabled.
    ///
    /// # Errors
    /// Returns an error if a drawer control never appears or the input is never delivered.
    pub async fn add_friend_through_ui(&self, addr: &str) -> Result<(), String> {
        self.press_key(NamedKey::Tab).await?;
        self.click(LobbyTarget::FriendAddressInput).await?;
        self.type_text(addr).await?;
        poll_until(
            &format!("{}: add-friend button enabled", self.name),
            self.deadlines.action(),
            async || self.locate(LobbyTarget::AddFriendButton).await,
            |found| found.is_some_and(|node| !node.disabled),
        )
        .await?;
        self.click(LobbyTarget::AddFriendButton).await
    }

    async fn press_key(&self, key: NamedKey) -> Result<(), String> {
        self.peer
            .call::<Value>("e2e/press_key", json!(key))
            .await
            .map_err(|error| self.peer.explain(&error))?;
        self.wait_until_input_idle().await
    }

    async fn type_text(&self, text: &str) -> Result<(), String> {
        self.peer
            .call::<Value>("e2e/type_text", json!(text))
            .await
            .map_err(|error| self.peer.explain(&error))?;
        self.wait_until_input_idle().await
    }

    async fn wait_until_input_idle(&self) -> Result<(), String> {
        poll_until(
            &format!("{}: input delivered", self.name),
            self.deadlines.action(),
            async || self.peer.call::<bool>("e2e/input_idle", Value::Null).await,
            |idle| *idle,
        )
        .await
        .map(|_| ())
    }

    async fn locate(&self, target: LobbyTarget) -> Result<Option<LocatedNode>, String> {
        self.peer.call("lobby/locate", json!(target)).await
    }

    async fn click(&self, target: LobbyTarget) -> Result<(), String> {
        let node = poll_until(
            &format!("{}: {target:?} on screen", self.name),
            self.deadlines.action(),
            async || self.locate(target.clone()).await,
            Option::is_some,
        )
        .await?
        .ok_or_else(|| format!("{}: {target:?} not on screen", self.name))?;
        let cursor = CursorTarget {
            x: node.centre.x,
            y: node.centre.y,
            speed: PointerSpeed(CLICK_SPEED),
        };
        self.peer
            .call::<Value>(
                "e2e/click_at",
                serde_json::to_value(cursor).map_err(|e| e.to_string())?,
            )
            .await
            .map(|_| ())
            .map_err(|error| self.peer.explain(&error))
    }

    /// # Errors
    /// Returns an error if the app does not exit in time or exits unsuccessfully.
    pub async fn exit(&mut self) -> Result<(), String> {
        self.peer.exit().await
    }

    /// Launches the same peer again with the same arguments, so it keeps its identity and book.
    /// The caller exits the earlier run first; its log stays readable through this wrapper.
    ///
    /// # Errors
    /// Returns an error if the app does not start or does not answer over BRP.
    pub async fn relaunch(&mut self) -> Result<(), String> {
        self.earlier_logs.push(self.peer.log_text());
        self.peer = Peer::launch(&self.bin, &self.args).await?;
        Ok(())
    }

    #[must_use]
    pub fn log_contains(&self, line: &str) -> bool {
        self.earlier_logs.iter().any(|log| log.contains(line)) || self.peer.log_contains(line)
    }
}
