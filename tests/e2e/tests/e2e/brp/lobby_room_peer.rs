use std::path::Path;

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

    /// Waits until the room has delivered at least one ping, and returns the sender peer ids.
    ///
    /// # Errors
    /// Returns an error if no ping arrives before the deadline.
    pub async fn wait_until_hears_a_peer(&self) -> Result<Vec<String>, String> {
        poll_until(
            "room ping received",
            self.lobby.deadlines().action(),
            async || self.messages().await,
            |senders| !senders.is_empty(),
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
}
