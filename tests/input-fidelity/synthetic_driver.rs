use std::time::Duration;

use freenet_libp2p_bevy_core::testing::harness::Polling;
use freenet_libp2p_bevy_core::testing::{CursorTarget, PointerSpeed};
use serde_json::Value;

use crate::input_script::CLICK_SPEED;
use crate::input_step::InputStep;
use crate::probe::ProbePeer;

pub struct SyntheticDriver<'a> {
    peer: &'a ProbePeer,
}

impl<'a> SyntheticDriver<'a> {
    #[must_use]
    pub const fn new(peer: &'a ProbePeer) -> Self {
        Self { peer }
    }

    /// # Errors
    /// Returns an error if any script step is not delivered in time.
    pub async fn play(&self, script: &[InputStep]) -> Result<(), String> {
        for step in script {
            match step {
                InputStep::ClickAt { x, y } => {
                    let target = CursorTarget {
                        x: *x,
                        y: *y,
                        speed: PointerSpeed(CLICK_SPEED),
                    };
                    let params = serde_json::to_value(target)
                        .map_err(|error| format!("synthetic driver: {error}"))?;
                    self.peer.call::<Value>("e2e/click_at", params).await?;
                }
                InputStep::MoveTo { x, y } => {
                    let target = CursorTarget {
                        x: *x,
                        y: *y,
                        speed: PointerSpeed(CLICK_SPEED),
                    };
                    let params = serde_json::to_value(target)
                        .map_err(|error| format!("synthetic driver: {error}"))?;
                    self.peer.call::<Value>("e2e/move_cursor", params).await?;
                }
                InputStep::Keys(step) => {
                    let params = serde_json::to_value(std::slice::from_ref(step))
                        .map_err(|error| format!("synthetic driver: {error}"))?;
                    self.peer.call::<Value>("e2e/keys", params).await?;
                }
            }
            self.wait_until_input_idle().await?;
        }
        Ok(())
    }

    async fn wait_until_input_idle(&self) -> Result<(), String> {
        Polling::from(Duration::from_secs(30))
            .until(
                "input delivered",
                async || self.peer.call::<bool>("e2e/input_idle", Value::Null).await,
                |idle| *idle,
            )
            .await
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use freenet_libp2p_bevy_core::testing::harness::E2EDisplay;

    use super::SyntheticDriver;
    use crate::input_script::INPUT_SCRIPT;
    use crate::probe::ProbePeer;

    const PROBE_BIN: &str = env!("CARGO_BIN_EXE_input_probe");

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "needs a private Xvfb and the lavapipe ICD; run with: just input-fidelity"]
    async fn synthetic_driver_plays_the_script() -> Result<(), String> {
        let run_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".local-run")
            .join(format!("input-fidelity-smoke-{}", std::process::id()));
        std::fs::create_dir_all(&run_dir).map_err(|error| error.to_string())?;
        let display = E2EDisplay::from_env(&run_dir)?
            .ok_or_else(|| "this smoke test only runs on Linux/X11".to_string())?;
        let probe = ProbePeer::try_from_launch(Path::new(PROBE_BIN), &display.name).await?;
        probe.wait_ready().await?;
        probe.clear().await?;
        SyntheticDriver::new(&probe).play(&INPUT_SCRIPT).await?;
        let records = probe.records().await?;
        probe.exit().await?;
        if records.is_empty() {
            return Err("the synthetic driver left no records in the probe".to_string());
        }
        Ok(())
    }
}
