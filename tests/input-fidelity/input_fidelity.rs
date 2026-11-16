use std::path::Path;
use std::time::Duration;

use freenet_libp2p_bevy_core::testing::harness::E2EDisplay;
use input_fidelity_e2e::canonical_input::CanonicalInput;
use input_fidelity_e2e::fidelity_report::FidelityReport;
use input_fidelity_e2e::input_record::InputRecord;

use crate::input_script::INPUT_SCRIPT;
use crate::probe::ProbePeer;
use crate::real_driver::RealDriver;
use crate::synthetic_driver::SyntheticDriver;

const PROBE_BIN: &str = env!("CARGO_BIN_EXE_input_probe");
const SETTLE: Duration = Duration::from_millis(700);

fn run_dir(tag: &str) -> Result<std::path::PathBuf, String> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".local-run")
        .join(format!("input-fidelity-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

async fn play_with_synthetic(bin: &Path, display: &str) -> Result<Vec<InputRecord>, String> {
    let probe = ProbePeer::try_from_launch(bin, display).await?;
    probe.wait_ready().await?;
    probe.clear().await?;
    SyntheticDriver::new(&probe).play(INPUT_SCRIPT).await?;
    tokio::time::sleep(SETTLE).await;
    let records = probe.records().await?;
    probe.exit().await?;
    Ok(records)
}

async fn play_with_real(bin: &Path, display: &str) -> Result<Vec<InputRecord>, String> {
    let probe = ProbePeer::try_from_launch(bin, display).await?;
    probe.wait_ready().await?;
    let geometry = probe.window().await?;
    let mut driver = RealDriver::try_from_geometry(display, &geometry)?;
    probe.wait_for_focus().await?;
    probe.clear().await?;
    driver.play(INPUT_SCRIPT)?;
    tokio::time::sleep(SETTLE).await;
    let records = probe.records().await?;
    probe.exit().await?;
    Ok(records)
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs a private Xvfb and the lavapipe ICD; run with: just input-fidelity"]
async fn synthetic_input_matches_real_os_input() -> Result<(), String> {
    let run_dir = run_dir("compare")?;
    let display = E2EDisplay::from_env(&run_dir)?
        .ok_or_else(|| "no X display: this test only runs on Linux/X11".to_string())?;
    let bin = Path::new(PROBE_BIN);
    let real = CanonicalInput::from_records(&play_with_real(bin, &display.name).await?);
    let synthetic = CanonicalInput::from_records(&play_with_synthetic(bin, &display.name).await?);
    let report = FidelityReport::between(real, synthetic);
    if report.is_match() {
        return Ok(());
    }
    Err(report.to_string())
}
