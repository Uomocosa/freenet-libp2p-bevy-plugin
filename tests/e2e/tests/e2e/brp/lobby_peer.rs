use std::time::Duration;

use freenet_libp2p_bevy_plugin::lobby_room::testing::{LobbyStatus, LobbyTarget, LocatedNode};
use freenet_libp2p_bevy_plugin::testing::{CursorTarget, PointerSpeed};
use serde_json::{Value, json};

use super::{Peer, poll_until};

const CLICK_SPEED: f32 = 1200.0;
const ACTION_DEADLINE: Duration = Duration::from_secs(60);
const LOBBY_DEADLINE: Duration = Duration::from_secs(120);

/// The lobby actions every app shares: create, join, membership, all through the default UI.
pub struct LobbyPeer {
    name: String,
    peer: Peer,
}

impl LobbyPeer {
    #[must_use]
    pub fn new(name: &str, peer: Peer) -> Self {
        Self {
            name: name.to_string(),
            peer,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn peer(&self) -> &Peer {
        &self.peer
    }

    /// # Errors
    /// Returns an error if the room cannot be created and joined.
    pub async fn create_room(&self, room: &str) -> Result<(), String> {
        self.attempt_create_room(room)
            .await
            .map_err(|error| self.peer.explain(&error))
    }

    async fn attempt_create_room(&self, room: &str) -> Result<(), String> {
        self.click(LobbyTarget::RoomNameInput).await?;
        self.peer
            .call::<Value>("e2e/type_text", json!(room))
            .await?;
        poll_until(
            &format!("{}: create button enabled", self.name),
            ACTION_DEADLINE,
            async || self.locate(LobbyTarget::CreateRoomButton).await,
            |found| found.is_some_and(|node| !node.disabled),
        )
        .await?;
        self.click(LobbyTarget::CreateRoomButton).await?;
        self.wait_until_joined(room, ACTION_DEADLINE).await
    }

    /// # Errors
    /// Returns an error if the room is never listed or the join does not complete.
    pub async fn join_room(&self, room: &str) -> Result<(), String> {
        self.attempt_join_room(room)
            .await
            .map_err(|error| self.peer.explain(&error))
    }

    async fn attempt_join_room(&self, room: &str) -> Result<(), String> {
        poll_until(
            &format!("{}: room {room} listed", self.name),
            LOBBY_DEADLINE,
            async || self.lobby_status().await,
            |status| status.rooms.iter().any(|listed| listed == room),
        )
        .await?;
        self.click(LobbyTarget::Room(room.to_string())).await?;
        self.wait_until_joined(room, ACTION_DEADLINE).await
    }

    async fn wait_until_joined(&self, room: &str, deadline: Duration) -> Result<(), String> {
        poll_until(
            &format!("{}: joined room {room}", self.name),
            deadline,
            async || self.lobby_status().await,
            |status| status.joined_room.as_deref() == Some(room),
        )
        .await?;
        Ok(())
    }

    /// Waits until the joined room lists `others` remote members (the peer itself is not counted).
    ///
    /// # Errors
    /// Returns an error if the remote member count never reaches `others`.
    pub async fn wait_until_room_has_other_members(&self, others: usize) -> Result<(), String> {
        let outcome = poll_until(
            &format!("{}: room lists {others} other members", self.name),
            LOBBY_DEADLINE,
            async || self.lobby_status().await,
            |status| status.other_members == others,
        )
        .await;
        outcome
            .map(|_| ())
            .map_err(|error| self.peer.explain(&error))
    }

    /// Names the peer, its last lobby status, and its filtered log tail, for failure messages.
    pub async fn describe(&self) -> String {
        let status = self.lobby_status().await;
        self.peer
            .explain(&format!("{}: lobby status {status:?}", self.name))
    }

    /// # Errors
    /// Returns an error if the lobby status cannot be read.
    pub async fn lobby_status(&self) -> Result<LobbyStatus, String> {
        self.peer.call("lobby/status", Value::Null).await
    }

    /// # Errors
    /// Returns an error if the app does not answer or does not exit cleanly.
    pub async fn exit(mut self) -> Result<(), String> {
        self.peer.exit().await
    }

    async fn locate(&self, target: LobbyTarget) -> Result<Option<LocatedNode>, String> {
        self.peer.call("lobby/locate", json!(target)).await
    }

    async fn click(&self, target: LobbyTarget) -> Result<(), String> {
        let node = poll_until(
            &format!("{}: {target:?} on screen", self.name),
            ACTION_DEADLINE,
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
            .await?;
        Ok(())
    }
}
