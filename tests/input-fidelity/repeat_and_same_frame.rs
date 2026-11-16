use std::path::Path;
use std::time::Duration;

use bevy::input::keyboard::KeyCode;
use freenet_libp2p_bevy_core::testing::harness::E2EDisplay;
use input_fidelity_e2e::input_record::{InputRecord, KeyLabel, PressState};

use crate::input_script::SYNTHETIC_ONLY_SCRIPT;
use crate::probe::ProbePeer;
use crate::synthetic_driver::SyntheticDriver;

const PROBE_BIN: &str = env!("CARGO_BIN_EXE_input_probe");

fn keyboard_sequence(records: &[InputRecord]) -> Vec<(KeyLabel, PressState, bool)> {
    let mut sequence = Vec::new();
    for record in records {
        if let InputRecord::Keyboard {
            physical,
            state,
            does_repeat,
            ..
        } = record
        {
            sequence.push((physical.clone(), *state, *does_repeat));
        }
    }
    sequence
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs a private Xvfb and the lavapipe ICD; run with: just input-fidelity"]
async fn synthetic_repeat_and_same_frame_are_recorded() -> Result<(), String> {
    let run_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".local-run")
        .join(format!(
            "input-fidelity-synthetic-only-{}",
            std::process::id()
        ));
    std::fs::create_dir_all(&run_dir).map_err(|error| error.to_string())?;
    let display = E2EDisplay::from_env(&run_dir)?
        .ok_or_else(|| "this synthetic-only test only runs on Linux/X11".to_string())?;
    let probe = ProbePeer::try_from_launch(Path::new(PROBE_BIN), &display.name).await?;
    probe.wait_ready().await?;
    probe.clear().await?;
    SyntheticDriver::new(&probe)
        .play(&SYNTHETIC_ONLY_SCRIPT)
        .await?;
    tokio::time::sleep(Duration::from_millis(700)).await;
    let records = probe.records().await?;
    probe.exit().await?;
    let arrow = KeyLabel::from_key_code(KeyCode::ArrowRight);
    let space = KeyLabel::from_key_code(KeyCode::Space);
    let sequence = keyboard_sequence(&records);
    let expected = vec![
        (arrow.clone(), PressState::Pressed, false),
        (arrow.clone(), PressState::Pressed, true),
        (arrow.clone(), PressState::Pressed, true),
        (arrow.clone(), PressState::Pressed, true),
        (arrow, PressState::Released, false),
        (space.clone(), PressState::Pressed, false),
        (space.clone(), PressState::Released, false),
        (space.clone(), PressState::Pressed, false),
        (space, PressState::Released, false),
    ];
    if sequence != expected {
        return Err(format!(
            "synthetic-only records differ:\n got {sequence:?}\n want {expected:?}"
        ));
    }
    let texts: Vec<Option<String>> = records
        .iter()
        .filter_map(|record| match record {
            InputRecord::Keyboard { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    let expected_texts: Vec<Option<String>> = vec![
        None,
        None,
        None,
        None,
        None,
        Some(" ".to_string()),
        Some(" ".to_string()),
        Some(" ".to_string()),
        Some(" ".to_string()),
    ];
    if texts != expected_texts {
        return Err(format!(
            "synthetic-only texts differ:\n got {texts:?}\n want {expected_texts:?}"
        ));
    }
    Ok(())
}
