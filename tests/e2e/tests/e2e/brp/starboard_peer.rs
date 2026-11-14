use std::path::Path;
use std::time::Duration;

use bevy::input::mouse::MouseButton;
use bevy::math::Vec2;
use e2e_apps::starboard::CanvasStatus;
use freenet_libp2p_bevy_plugin::testing::{PointerPath, PointerSpeed};
use serde_json::{Value, json};

use super::{BrpNetwork, LobbyPeer, Peer, Stroke, poll_until};

const HAND_SPEED: f32 = 600.0;

/// One `starboard_e2e` process: the lobby actions plus painting and the canvas status.
pub struct StarboardPeer {
    lobby: LobbyPeer,
}

impl StarboardPeer {
    /// # Errors
    /// Returns an error if the peer does not start or does not answer over BRP.
    pub async fn launch(
        bin: &Path,
        name: &str,
        net: &BrpNetwork,
        window: (i32, i32),
    ) -> Result<Self, String> {
        let args = net.peer_args(name, window);
        let peer = Peer::launch(bin, &args).await?;
        Ok(Self {
            lobby: LobbyPeer::new(name, peer, net.deadlines()),
        })
    }

    /// Launches a peer that presents `token` instead of the run's shared token.
    ///
    /// # Errors
    /// Returns an error if the peer does not start or does not answer over BRP.
    pub async fn launch_with_token(
        bin: &Path,
        name: &str,
        net: &BrpNetwork,
        window: (i32, i32),
        token: &str,
    ) -> Result<Self, String> {
        let args = net.peer_args_with_token(name, window, token);
        let peer = Peer::launch(bin, &args).await?;
        Ok(Self {
            lobby: LobbyPeer::new(name, peer, net.deadlines()),
        })
    }

    /// # Errors
    /// Returns an error if the room cannot be created and joined.
    pub async fn create_room(&self, room: &str) -> Result<(), String> {
        self.lobby.create_room(room).await
    }

    /// # Errors
    /// Returns an error if the room is never listed.
    pub async fn wait_until_room_listed(&self, room: &str) -> Result<(), String> {
        self.lobby.wait_until_room_listed(room).await
    }

    /// # Errors
    /// Returns an error if the join succeeds, or no failure is recorded before the deadline.
    pub async fn join_room_expecting_failure(&self, room: &str) -> Result<String, String> {
        self.lobby.join_room_expecting_failure(room).await
    }

    /// # Errors
    /// Returns an error if the room is never listed or the join does not complete.
    pub async fn join_room(&self, room: &str) -> Result<(), String> {
        self.lobby.join_room(room).await
    }

    /// # Errors
    /// Returns an error if the remote member count never reaches `others`.
    pub async fn wait_until_room_has_other_members(&self, others: usize) -> Result<(), String> {
        self.lobby.wait_until_room_has_other_members(others).await
    }

    /// # Errors
    /// Returns an error if the remote member count never reaches `others` or more.
    pub async fn wait_until_room_has_at_least_other_members(
        &self,
        others: usize,
    ) -> Result<(), String> {
        self.lobby
            .wait_until_room_has_at_least_other_members(others)
            .await
    }

    /// # Errors
    /// Returns an error if the remote member count never drops to `others` or fewer.
    pub async fn wait_until_room_has_at_most_other_members(
        &self,
        others: usize,
    ) -> Result<(), String> {
        self.lobby
            .wait_until_room_has_at_most_other_members(others)
            .await
    }

    /// # Errors
    /// Returns an error if the app cannot be killed.
    pub fn kill(&mut self) -> Result<(), String> {
        self.lobby.kill()
    }

    /// # Errors
    /// Returns an error if any stroke cannot be painted.
    pub async fn paint_all(&self, strokes: &[Stroke]) -> Result<(), String> {
        for &stroke in strokes {
            self.paint(stroke).await?;
        }
        Ok(())
    }

    /// # Errors
    /// Returns an error if any stroke cannot be erased.
    pub async fn erase_all(&self, strokes: &[Stroke]) -> Result<(), String> {
        for &stroke in strokes {
            self.erase(stroke).await?;
        }
        Ok(())
    }

    /// # Errors
    /// Returns an error if the stroke cannot be mapped or the input does not finish.
    pub async fn paint(&self, stroke: Stroke) -> Result<(), String> {
        self.follow(MouseButton::Left, stroke)
            .await
            .map_err(|error| self.lobby.peer().explain(&error))
    }

    /// Erases with the right button, which `track_pointer` maps to erase.
    ///
    /// # Errors
    /// Returns an error if the stroke cannot be mapped or the input does not finish.
    pub async fn erase(&self, stroke: Stroke) -> Result<(), String> {
        self.follow(MouseButton::Right, stroke)
            .await
            .map_err(|error| self.lobby.peer().explain(&error))
    }

    async fn follow(&self, button: MouseButton, stroke: Stroke) -> Result<(), String> {
        let peer = self.lobby.peer();
        let points: Vec<Vec2> = peer
            .call("starboard/board_to_window", json!(stroke.0))
            .await?;
        let path = PointerPath {
            button,
            points,
            speed: PointerSpeed(HAND_SPEED),
        };
        peer.call::<Value>(
            "e2e/follow",
            serde_json::to_value(path).map_err(|e| e.to_string())?,
        )
        .await?;
        poll_until(
            &format!("{}: input delivered", self.name()),
            self.lobby.deadlines().action(),
            async || peer.call::<bool>("e2e/input_idle", Value::Null).await,
            |idle| *idle,
        )
        .await?;
        Ok(())
    }

    fn name(&self) -> &str {
        self.lobby.name()
    }

    /// # Errors
    /// Returns an error if the canvas never gains a painted tile.
    pub async fn wait_until_canvas_has_content(
        &self,
        deadline: Duration,
    ) -> Result<CanvasStatus, String> {
        let outcome = poll_until(
            &format!("{}: canvas has content", self.name()),
            deadline,
            async || self.canvas_status().await,
            |status| status.painted_tiles > 0,
        )
        .await;
        outcome.map_err(|error| self.lobby.peer().explain(&error))
    }

    /// # Errors
    /// Returns an error if the canvas keeps changing for the whole deadline.
    pub async fn wait_until_canvas_stops_changing(
        &self,
        quiet: Duration,
    ) -> Result<CanvasStatus, String> {
        poll_until(
            &format!("{}: canvas stable for {quiet:?}", self.name()),
            self.lobby.deadlines().action(),
            async || self.canvas_status().await,
            |status| u128::from(status.stable_for_ms) >= quiet.as_millis(),
        )
        .await
    }

    /// # Errors
    /// Returns an error if the cell never gains a paint colour.
    pub async fn wait_until_cell_is_painted(&self, cell: (u16, u16)) -> Result<(), String> {
        poll_until(
            &format!("{}: cell {cell:?} painted", self.name()),
            self.lobby.deadlines().action(),
            async || self.cell_painted(cell).await,
            |painted| *painted,
        )
        .await
        .map(|_| ())
        .map_err(|error| self.lobby.peer().explain(&error))
    }

    async fn cell_painted(&self, cell: (u16, u16)) -> Result<bool, String> {
        self.lobby
            .peer()
            .call("starboard/cell_painted", json!(cell))
            .await
    }

    /// # Errors
    /// Returns an error if the request fails.
    pub async fn canvas_status(&self) -> Result<CanvasStatus, String> {
        self.lobby
            .peer()
            .call("starboard/status", Value::Null)
            .await
    }

    /// Names the peer, its last lobby status, and its filtered log tail, for failure messages.
    pub async fn describe(&self) -> String {
        self.lobby.describe().await
    }

    /// # Errors
    /// Returns an error if the app does not answer or does not exit cleanly.
    pub async fn exit(self) -> Result<(), String> {
        self.lobby.exit().await
    }
}
