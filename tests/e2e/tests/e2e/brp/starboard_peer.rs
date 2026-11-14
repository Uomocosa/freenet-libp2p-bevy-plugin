use std::path::Path;
use std::time::Duration;

use bevy::input::mouse::MouseButton;
use bevy::math::Vec2;
use e2e_apps::starboard::CanvasStatus;
use freenet_libp2p_bevy_plugin::lobby_room::testing::{LobbyStatus, LobbyTarget, LocatedNode};
use freenet_libp2p_bevy_plugin::testing::{CursorTarget, PointerPath, PointerSpeed};
use serde_json::{Value, json};

use super::{Peer, Stroke, poll_until};

const HAND_SPEED: f32 = 600.0;
const CLICK_SPEED: f32 = 1200.0;
const ACTION_DEADLINE: Duration = Duration::from_secs(60);
const LOBBY_DEADLINE: Duration = Duration::from_secs(120);

pub struct StarboardPeer {
    name: String,
    peer: Peer,
}

impl StarboardPeer {
    /// # Errors
    /// Returns an error if the peer does not start or does not answer over BRP.
    pub async fn launch(
        bin: &Path,
        name: &str,
        token: &str,
        orchestrator_port: u16,
    ) -> Result<Self, String> {
        let args = [
            "--username".to_string(),
            name.to_string(),
            "--token".to_string(),
            token.to_string(),
            "--network".to_string(),
            "fake".to_string(),
            "--orchestrator-port".to_string(),
            orchestrator_port.to_string(),
        ];
        let peer = Peer::launch(bin, &args).await?;
        Ok(Self {
            name: name.to_string(),
            peer,
        })
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
            |status| status.members == others,
        )
        .await;
        outcome
            .map(|_| ())
            .map_err(|error| self.peer.explain(&error))
    }

    /// # Errors
    /// Returns an error if the stroke cannot be mapped or the input does not finish.
    pub async fn paint(&self, stroke: Stroke) -> Result<(), String> {
        self.follow(MouseButton::Left, stroke)
            .await
            .map_err(|error| self.peer.explain(&error))
    }

    /// Erases with the right button, which `track_pointer` maps to erase.
    ///
    /// # Errors
    /// Returns an error if the stroke cannot be mapped or the input does not finish.
    pub async fn erase(&self, stroke: Stroke) -> Result<(), String> {
        self.follow(MouseButton::Right, stroke)
            .await
            .map_err(|error| self.peer.explain(&error))
    }

    async fn follow(&self, button: MouseButton, stroke: Stroke) -> Result<(), String> {
        let points: Vec<Vec2> = self
            .peer
            .call("starboard/board_to_window", json!(stroke.0))
            .await?;
        let path = PointerPath {
            button,
            points,
            speed: PointerSpeed(HAND_SPEED),
        };
        self.peer
            .call::<Value>(
                "e2e/follow",
                serde_json::to_value(path).map_err(|e| e.to_string())?,
            )
            .await?;
        poll_until(
            &format!("{}: input delivered", self.name),
            ACTION_DEADLINE,
            async || self.peer.call::<bool>("e2e/input_idle", Value::Null).await,
            |idle| *idle,
        )
        .await?;
        Ok(())
    }

    /// # Errors
    /// Returns an error if the canvas never gains a painted tile.
    pub async fn wait_until_canvas_has_content(
        &self,
        deadline: Duration,
    ) -> Result<CanvasStatus, String> {
        let outcome = poll_until(
            &format!("{}: canvas has content", self.name),
            deadline,
            async || self.canvas_status().await,
            |status| status.painted_tiles > 0,
        )
        .await;
        outcome.map_err(|error| self.peer.explain(&error))
    }

    /// # Errors
    /// Returns an error if the canvas keeps changing for the whole deadline.
    pub async fn wait_until_canvas_stops_changing(
        &self,
        quiet: Duration,
    ) -> Result<CanvasStatus, String> {
        poll_until(
            &format!("{}: canvas stable for {quiet:?}", self.name),
            ACTION_DEADLINE,
            async || self.canvas_status().await,
            |status| u128::from(status.stable_for_ms) >= quiet.as_millis(),
        )
        .await
    }

    /// # Errors
    /// Returns an error if the request fails.
    pub async fn canvas_status(&self) -> Result<CanvasStatus, String> {
        self.peer.call("starboard/status", Value::Null).await
    }

    /// # Errors
    /// Returns an error if the app does not answer or does not exit cleanly.
    pub async fn exit(mut self) -> Result<(), String> {
        self.peer.exit().await
    }

    /// Names the peer, its last lobby status, and its filtered log tail, for failure messages.
    pub async fn describe(&self) -> String {
        let status = self.lobby_status().await;
        self.peer
            .explain(&format!("{}: lobby status {status:?}", self.name))
    }

    async fn lobby_status(&self) -> Result<LobbyStatus, String> {
        self.peer.call("lobby/status", Value::Null).await
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
