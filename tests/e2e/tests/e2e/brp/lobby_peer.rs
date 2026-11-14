use std::time::Duration;

use freenet_libp2p_bevy_plugin::lobby_room::testing::{LobbyStatus, LobbyTarget, LocatedNode};
use freenet_libp2p_bevy_plugin::testing::{CursorTarget, NamedKey, PointerSpeed};
use serde_json::{Value, json};

use super::{Deadlines, Peer, poll_until};

const CLICK_SPEED: f32 = 1200.0;
const JOIN_FAILURE_BUDGET: Duration = Duration::from_secs(180);

/// The lobby actions every app shares: create, join, membership, all through the default UI.
pub struct LobbyPeer {
    name: String,
    peer: Peer,
    deadlines: Deadlines,
}

impl LobbyPeer {
    #[must_use]
    pub fn new(name: &str, peer: Peer, deadlines: Deadlines) -> Self {
        Self {
            name: name.to_string(),
            peer,
            deadlines,
        }
    }

    #[must_use]
    pub const fn deadlines(&self) -> Deadlines {
        self.deadlines
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

    /// Types `room` with one extra character, presses Backspace to delete it, then creates the
    /// room: the joined room must be `room`, so the deletion reached the room name.
    ///
    /// # Errors
    /// Returns an error if the room cannot be created and joined under the name `room`.
    pub async fn create_room_after_backspace(&self, room: &str) -> Result<(), String> {
        self.attempt_create_room_after_backspace(room)
            .await
            .map_err(|error| self.peer.explain(&error))
    }

    async fn attempt_create_room(&self, room: &str) -> Result<(), String> {
        self.click(LobbyTarget::RoomNameInput).await?;
        self.type_text(room).await?;
        self.submit_room(room).await
    }

    async fn attempt_create_room_after_backspace(&self, room: &str) -> Result<(), String> {
        self.click(LobbyTarget::RoomNameInput).await?;
        self.type_text(&format!("{room}x")).await?;
        self.press_key(NamedKey::Backspace).await?;
        self.submit_room(room).await
    }

    async fn submit_room(&self, room: &str) -> Result<(), String> {
        poll_until(
            &format!("{}: create button enabled", self.name),
            self.deadlines.action(),
            async || self.locate(LobbyTarget::CreateRoomButton).await,
            |found| found.is_some_and(|node| !node.disabled),
        )
        .await?;
        self.click(LobbyTarget::CreateRoomButton).await?;
        self.wait_until_joined(room, self.deadlines.action()).await
    }

    async fn type_text(&self, text: &str) -> Result<(), String> {
        self.peer
            .call::<Value>("e2e/type_text", json!(text))
            .await?;
        self.wait_until_input_idle().await
    }

    async fn press_key(&self, key: NamedKey) -> Result<(), String> {
        self.peer.call::<Value>("e2e/press_key", json!(key)).await?;
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

    /// # Errors
    /// Returns an error if the room is never listed or the join does not complete.
    pub async fn join_room(&self, room: &str) -> Result<(), String> {
        self.attempt_join_room(room)
            .await
            .map_err(|error| self.peer.explain(&error))
    }

    async fn attempt_join_room(&self, room: &str) -> Result<(), String> {
        self.await_room_listed(room).await?;
        self.click(LobbyTarget::Room(room.to_string())).await?;
        self.wait_until_joined(room, self.deadlines.action()).await
    }

    /// Waits until the lobby lists `room`.
    ///
    /// # Errors
    /// Returns an error if the room is never listed.
    pub async fn wait_until_room_listed(&self, room: &str) -> Result<(), String> {
        self.await_room_listed(room)
            .await
            .map_err(|error| self.peer.explain(&error))
    }

    async fn await_room_listed(&self, room: &str) -> Result<(), String> {
        poll_until(
            &format!("{}: room {room} listed", self.name),
            self.deadlines.lobby(),
            async || self.lobby_status().await,
            |status| status.rooms.iter().any(|listed| listed == room),
        )
        .await
        .map(|_| ())
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
            self.deadlines.lobby(),
            async || self.lobby_status().await,
            |status| status.other_members == others,
        )
        .await;
        outcome
            .map(|_| ())
            .map_err(|error| self.peer.explain(&error))
    }

    /// # Errors
    /// Returns an error if the remote member count never reaches `others` or more.
    pub async fn wait_until_room_has_at_least_other_members(
        &self,
        others: usize,
    ) -> Result<(), String> {
        self.wait_until_other_members(&format!("at least {others}"), |count| count >= others)
            .await
    }

    /// # Errors
    /// Returns an error if the remote member count never drops to `others` or fewer.
    pub async fn wait_until_room_has_at_most_other_members(
        &self,
        others: usize,
    ) -> Result<(), String> {
        self.wait_until_other_members(&format!("at most {others}"), |count| count <= others)
            .await
    }

    async fn wait_until_other_members(
        &self,
        bound: &str,
        done: impl Fn(usize) -> bool,
    ) -> Result<(), String> {
        poll_until(
            &format!("{}: room lists {bound} other members", self.name),
            self.deadlines.lobby(),
            async || self.lobby_status().await,
            |status| done(status.other_members),
        )
        .await
        .map(|_| ())
        .map_err(|error| self.peer.explain(&error))
    }

    /// Clicks `room` expecting the join to fail, and returns the reason.
    ///
    /// The failure is the one `join_failure_reason` applies: a recorded `join_error`, or no room
    /// joined while the fetched lobby does not list `room`. A room that is not listed at all is
    /// such a failure as soon as the lobby is fetched, since the room cannot be clicked.
    ///
    /// # Errors
    /// Returns an error if no failure is recorded before the deadline.
    pub async fn join_room_expecting_failure(&self, room: &str) -> Result<String, String> {
        self.attempt_join_expecting_failure(room)
            .await
            .map_err(|error| self.peer.explain(&error))
    }

    async fn attempt_join_expecting_failure(&self, room: &str) -> Result<String, String> {
        let fetched = poll_until(
            &format!("{}: lobby fetched", self.name),
            self.deadlines.lobby(),
            async || self.lobby_status().await,
            |status| status.lobby_fetched,
        )
        .await?;
        if !fetched.rooms.iter().any(|listed| listed == room) {
            return Ok(format!("room {room} is not listed"));
        }
        self.click(LobbyTarget::Room(room.to_string())).await?;
        let outcome = poll_until(
            &format!("{}: join of {room} fails", self.name),
            self.deadlines.scaled(JOIN_FAILURE_BUDGET),
            async || self.lobby_status().await,
            |status| join_failure(status, room).is_some(),
        )
        .await?;
        join_failure(&outcome, room)
            .ok_or_else(|| format!("{}: join of {room} ended without a failure", self.name))
    }

    /// Waits until the lobby has been fetched and no longer lists `room`.
    ///
    /// # Errors
    /// Returns an error if the room is still listed at the deadline.
    pub async fn wait_until_room_gone(&self, room: &str) -> Result<(), String> {
        poll_until(
            &format!("{}: room {room} gone", self.name),
            self.deadlines.lobby(),
            async || self.lobby_status().await,
            |status| status.lobby_fetched && !status.rooms.iter().any(|listed| listed == room),
        )
        .await
        .map(|_| ())
        .map_err(|error| self.peer.explain(&error))
    }

    /// Leaves the joined room through the Esc menu: Escape, the menu's Leave button, then Yes.
    ///
    /// # Errors
    /// Returns an error if a leave control never appears or the room stays joined.
    pub async fn leave_room(&self) -> Result<(), String> {
        self.attempt_leave_room()
            .await
            .map_err(|error| self.peer.explain(&error))
    }

    async fn attempt_leave_room(&self) -> Result<(), String> {
        self.press_key(NamedKey::Escape).await?;
        self.click(LobbyTarget::EscMenuLeaveButton).await?;
        self.click(LobbyTarget::LeaveYesButton).await?;
        poll_until(
            &format!("{}: left the joined room", self.name),
            self.deadlines.action(),
            async || self.lobby_status().await,
            |status| status.joined_room.is_none(),
        )
        .await
        .map(|_| ())
    }

    /// # Errors
    /// Returns an error if the app cannot be killed.
    pub fn kill(&mut self) -> Result<(), String> {
        self.peer.kill()
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
            .await?;
        Ok(())
    }
}

/// The failure a join of `room` has recorded: `join_error`, or no room joined while the fetched
/// lobby does not list `room`. Mirrors `LobbyStep::join_failure_reason`.
fn join_failure(status: &LobbyStatus, room: &str) -> Option<String> {
    if let Some(error) = &status.join_error {
        return Some(error.clone());
    }
    let gone = status.joined_room.is_none()
        && status.lobby_fetched
        && !status.rooms.iter().any(|listed| listed == room);
    gone.then(|| format!("room {room} no longer exists"))
}
