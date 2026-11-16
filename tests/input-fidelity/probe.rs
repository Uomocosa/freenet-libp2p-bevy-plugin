use std::path::Path;
use std::time::Duration;

use freenet_libp2p_bevy_core::testing::harness::{Peer, Polling};
use input_fidelity_e2e::input_record::InputRecord;
use input_fidelity_e2e::window_geometry::WindowGeometry;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub struct ProbePeer {
    peer: Peer,
}

impl ProbePeer {
    /// # Errors
    /// Returns an error if the probe binary does not start or does not answer over BRP.
    pub async fn try_from_launch(bin: &Path, display: &str) -> Result<Self, String> {
        let args: [String; 0] = [];
        let peer = Peer::try_from_launch(bin, &args, Some(display))
            .await
            .map_err(|error| format!("probe launch: {error}"))?;
        Ok(Self { peer })
    }

    /// # Errors
    /// Returns an error if the probe app does not answer or the call times out.
    pub async fn call<T: DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, String> {
        self.peer
            .call(method, params)
            .await
            .map_err(|error| self.peer.explain(&error))
    }

    /// # Errors
    /// Returns an error if the probe never reports enough frames in time.
    pub async fn wait_ready(&self) -> Result<(), String> {
        Polling::from(Duration::from_secs(30))
            .until(
                "probe ready",
                async || self.call::<bool>("probe/ready", Value::Null).await,
                |ready| *ready,
            )
            .await
            .map(|_| ())
    }

    /// # Errors
    /// Returns an error if the probe does not answer.
    pub async fn clear(&self) -> Result<(), String> {
        self.call::<Value>("probe/clear", Value::Null)
            .await
            .map(|_| ())
    }

    /// # Errors
    /// Returns an error if the probe does not answer.
    pub async fn records(&self) -> Result<Vec<InputRecord>, String> {
        self.call("probe/records", Value::Null).await
    }

    /// # Errors
    /// Returns an error if the probe does not answer.
    pub async fn window(&self) -> Result<WindowGeometry, String> {
        self.call("probe/window", Value::Null).await
    }

    /// # Errors
    /// Returns an error if the probe does not exit cleanly.
    pub async fn exit(mut self) -> Result<(), String> {
        self.peer.exit().await
    }
}
