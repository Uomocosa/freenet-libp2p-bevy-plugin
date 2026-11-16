use freenet_libp2p_bevy_plugin::lobby_room::testing::{LobbyTarget, LocatedNode};
use freenet_libp2p_bevy_plugin::testing::harness::KeysBuilder;
use freenet_libp2p_bevy_plugin::testing::{CursorTarget, Key, PointerSpeed};
use serde_json::{Value, json};

use super::{DayMillis, Deadlines, Peer, poll_until};

const CLICK_SPEED: f32 = 1200.0;

/// The default-UI inputs every app shares (click, type, key, locate) against one running peer.
/// Errors carry the app's log tail through `Peer::explain`.
pub struct UiPeer<'a> {
    name: &'a str,
    peer: &'a Peer,
    deadlines: Deadlines,
}

impl<'a> UiPeer<'a> {
    #[must_use]
    pub const fn new(name: &'a str, peer: &'a Peer, deadlines: Deadlines) -> Self {
        Self {
            name,
            peer,
            deadlines,
        }
    }

    #[must_use]
    pub const fn keys(&self) -> KeysBuilder<'_> {
        self.peer.keys(self.deadlines)
    }

    /// # Errors
    /// Returns an error if the key is not delivered in time.
    pub async fn press_key(&self, key: Key) -> Result<(), String> {
        self.keys().tap(key).send().await
    }

    /// # Errors
    /// Returns an error if the text is not delivered in time.
    pub async fn type_text(&self, text: &str) -> Result<(), String> {
        self.keys().text(text).send().await
    }

    /// # Errors
    /// Returns an error if the app does not answer.
    pub async fn locate(&self, target: LobbyTarget) -> Result<Option<LocatedNode>, String> {
        self.peer.call("lobby/locate", json!(target)).await
    }

    /// Clicks `target` once it is on screen; returns the press time of day in milliseconds.
    ///
    /// # Errors
    /// Returns an error if the target never shows up or the click is not delivered.
    pub async fn click(&self, target: LobbyTarget) -> Result<u64, String> {
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
        let pressed = DayMillis::now();
        self.peer
            .call::<Value>(
                "e2e/click_at",
                serde_json::to_value(cursor).map_err(|e| e.to_string())?,
            )
            .await
            .map_err(|error| self.peer.explain(&error))?;
        Ok(pressed)
    }
}
