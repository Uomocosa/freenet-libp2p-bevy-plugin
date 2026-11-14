use std::path::Path;
use std::time::Duration;

use freenet_libp2p_bevy_plugin::lobby_room::testing::LobbyStatus;
use serde_json::json;

use super::{BrpNetwork, LobbyPeer, Peer, poll_until};

/// One `lobby_room_e2e` process: the example app with the default lobby UI and room pings on.
pub struct LobbyRoomPeer {
    lobby: LobbyPeer,
}

impl LobbyRoomPeer {
    /// # Errors
    /// Returns an error if the peer does not start or does not answer over BRP.
    pub async fn launch(
        bin: &Path,
        name: &str,
        net: &BrpNetwork,
        window: (i32, i32),
    ) -> Result<Self, String> {
        let mut args = net.peer_args(name, window);
        args.push("--room-ping".to_string());
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

    /// Creates `room` and returns the time (day millis, UTC) its create button was pressed.
    ///
    /// # Errors
    /// Returns an error if the room cannot be created and joined.
    pub async fn create_room_timed(&self, room: &str) -> Result<u64, String> {
        self.lobby.create_room_timed(room).await
    }

    /// # Errors
    /// Returns an error if the room cannot be created under the name `room`.
    pub async fn create_room_after_backspace(&self, room: &str) -> Result<(), String> {
        self.lobby.create_room_after_backspace(room).await
    }

    /// # Errors
    /// Returns an error if the lobby status cannot be read.
    pub async fn lobby_status(&self) -> Result<LobbyStatus, String> {
        self.lobby.lobby_status().await
    }

    /// # Errors
    /// Returns an error if the room is never listed or the join does not complete.
    pub async fn join_room(&self, room: &str) -> Result<(), String> {
        self.lobby.join_room(room).await
    }

    /// Joins `room` and returns the time (day millis, UTC) its room button was pressed.
    ///
    /// # Errors
    /// Returns an error if the room is never listed or the join does not complete.
    pub async fn join_room_timed(&self, room: &str) -> Result<u64, String> {
        self.lobby.join_room_timed(room).await
    }

    /// Waits until the lobby lists `room`.
    ///
    /// # Errors
    /// Returns an error if the room is never listed.
    pub async fn wait_until_room_listed(&self, room: &str) -> Result<(), String> {
        self.lobby.wait_until_room_listed(room).await
    }

    /// Waits until the joined room lists at least `others` remote members.
    ///
    /// # Errors
    /// Returns an error if the remote member count never reaches `others`.
    pub async fn wait_until_room_has_at_least_other_members(
        &self,
        others: usize,
    ) -> Result<(), String> {
        self.lobby
            .wait_until_room_has_at_least_other_members(others)
            .await
    }

    /// Waits until the lobby has been fetched, as the `wait-lobby-ready` step does.
    ///
    /// # Errors
    /// Returns an error if the lobby is never fetched before `deadline`.
    pub async fn wait_until_lobby_fetched(&self, deadline: Duration) -> Result<(), String> {
        poll_until(
            "lobby fetched",
            deadline,
            async || self.lobby_status().await,
            |status| status.lobby_fetched,
        )
        .await
        .map(|_| ())
    }

    /// Waits until the embedded Freenet node reports a ring peer, as the `wait-ring-joined` step does.
    ///
    /// # Errors
    /// Returns an error if the node reports no ring peer before `deadline`.
    pub async fn wait_until_ring_joined(&self, deadline: Duration) -> Result<(), String> {
        poll_until(
            "ring joined",
            deadline,
            async || self.ring_peers().await,
            |peers| *peers > 0,
        )
        .await
        .map(|_| ())
        .map_err(|error| self.describe_error(&error))
    }

    /// The ring peers the embedded Freenet node reports.
    ///
    /// # Errors
    /// Returns an error if the request fails.
    pub async fn ring_peers(&self) -> Result<usize, String> {
        self.lobby.peer().call("lobby_room/ring", json!(null)).await
    }

    /// The path of the app's log file, readable until this peer is dropped.
    #[must_use]
    pub fn log_path(&self) -> &Path {
        self.lobby.peer().log_path()
    }

    /// # Errors
    /// Returns an error if the remote member count never reaches `others`.
    pub async fn wait_until_room_has_other_members(&self, others: usize) -> Result<(), String> {
        self.lobby.wait_until_room_has_other_members(others).await
    }

    /// Waits until the room has delivered at least one ping, and returns the sender peer ids.
    ///
    /// # Errors
    /// Returns an error if no ping arrives before the deadline.
    pub async fn wait_until_hears_a_peer(&self) -> Result<Vec<String>, String> {
        self.wait_until_heard_from(1, self.lobby.deadlines().action())
            .await
    }

    /// Waits until the room has delivered pings from at least `senders` distinct peers.
    ///
    /// # Errors
    /// Returns an error if too few senders are heard before `deadline`.
    pub async fn wait_until_heard_from(
        &self,
        senders: usize,
        deadline: Duration,
    ) -> Result<Vec<String>, String> {
        poll_until(
            "room pings received",
            deadline,
            async || self.messages().await,
            |heard| heard.len() >= senders,
        )
        .await
    }

    /// # Errors
    /// Returns an error if the request fails.
    pub async fn messages(&self) -> Result<Vec<String>, String> {
        self.lobby
            .peer()
            .call("lobby_room/messages", json!(null))
            .await
    }

    /// # Errors
    /// Returns an error if the app does not answer or does not exit cleanly.
    pub async fn exit(self) -> Result<(), String> {
        self.lobby.exit().await
    }

    pub async fn describe(&self) -> String {
        self.lobby.describe().await
    }

    fn describe_error(&self, error: &str) -> String {
        self.lobby.peer().explain(error)
    }
}
