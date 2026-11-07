use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use freenet_libp2p_bevy_plugin::testing::{Orchestrator, OrchestratorReport, ReportCheck};

use crate::check_layer::CheckLayer;
use crate::peer_run_result::PeerRunResult;
use crate::scenario_expectation::ScenarioExpectation;

/// How long ffmpeg may record before it stops itself. The run stops the
/// recording early once every peer's script has ended.
const RECORD_SECS: u64 = 3600;

/// How often the tiling loop repositions the peer windows while they appear.
const TILE_EVERY: std::time::Duration = std::time::Duration::from_secs(2);

/// Fails the test unless every check in the report passed.
pub trait AssertAll {
    fn assert_all(&self);
}

impl AssertAll for OrchestratorReport {
    fn assert_all(&self) {
        for check in &self.checks {
            assert!(
                check.ok,
                "check `{}` failed: {} (logs: {})",
                check.name,
                check.detail,
                self.logs_dir.display()
            );
        }
    }
}

/// Runs one configured `Orchestrator` on screen: wakes the display, records it,
/// tiles the peer windows as they appear, runs the peers to completion, stops
/// the recording into a sped-up clip, appends the `result.json` expectation
/// rows, and best-effort sends the clip unless Telegram has no credentials.
pub async fn run_with_recording(orchestrator: &mut Orchestrator) -> OrchestratorReport {
    crate::wakeup_screen::wakeup_screen();
    if let Err(detail) = crate::require_x11::require_x11() {
        return OrchestratorReport {
            checks: vec![ReportCheck {
                name: "setup".to_string(),
                ok: false,
                detail,
            }],
            logs_dir: PathBuf::new(),
            caption: "e2e setup failed".to_string(),
        };
    }
    crate::cleanup_stale::cleanup_stale();
    let peer_names = orchestrator.peer_names();
    let check_canvas = orchestrator.should_check_canvas_convergence;
    let check_history = orchestrator.should_check_history_written;
    let min_members = orchestrator
        .min_members_override
        .unwrap_or(peer_names.len());
    let may_crash = orchestrator.may_crash.clone();
    let join_must_fail = orchestrator.join_must_fail.clone();
    let stamp = epoch_secs();
    let raw = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".local-run")
        .join(format!("e2e-{stamp}-raw.mp4"));
    let recording = crate::start_record::start_record(RECORD_SECS, &raw);
    // The window titles come from the example (`starboard-{username}`); this
    // harness only runs starboard scenarios in this slice.
    let titles: Vec<String> = peer_names
        .iter()
        .map(|name| format!("starboard-{name}"))
        .collect();
    let tiler = tokio::spawn(async move {
        loop {
            let borrowed: Vec<&str> = titles.iter().map(String::as_str).collect();
            if let Err(error) = crate::tile::tile(&borrowed) {
                eprintln!("tile: {error}");
            }
            tokio::time::sleep(TILE_EVERY).await;
        }
    });
    let mut report = orchestrator.run().await;
    tiler.abort();
    let skip_lobby_fetch = orchestrator.should_forbid_freenet_node;
    let clip = if report.logs_dir.as_os_str().is_empty() {
        None
    } else {
        recording
            .and_then(|child| crate::finish_record::finish_record(child, &raw))
            .and_then(|raw| crate::speed_clip::speed_clip(&raw, &report.logs_dir.join("clip.mp4")))
    };
    append_expectations(
        &mut report,
        &peer_names,
        check_canvas,
        check_history,
        min_members,
        skip_lobby_fetch,
        may_crash,
        join_must_fail,
    );
    eprintln!("{}", report.caption);
    send_clip(clip.as_deref(), &report.caption).await;
    report
}

// needed helper: appends the result.json expectation rows for one run
fn append_expectations(
    report: &mut OrchestratorReport,
    peer_names: &[String],
    check_canvas: bool,
    check_history: bool,
    min_members: usize,
    skip_lobby_fetch: bool,
    may_crash: BTreeSet<String>,
    join_must_fail: BTreeSet<String>,
) {
    if peer_names.is_empty() || report.logs_dir.as_os_str().is_empty() {
        return;
    }
    let mut results: BTreeMap<String, Option<PeerRunResult>> = BTreeMap::new();
    for name in peer_names {
        let path = report.logs_dir.join(name).join("result.json");
        let parsed = std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<PeerRunResult>(&bytes).ok());
        results.insert(name.clone(), parsed);
    }
    let expectation = ScenarioExpectation {
        canvas_converges: check_canvas,
        min_members,
        may_crash,
        join_must_fail,
    };
    let kept: Vec<crate::expectation_result::ExpectationResult> = expectation
        .check_peer_run_results(&results)
        .into_iter()
        .filter(|row| !(skip_lobby_fetch && row.layer == CheckLayer::Freenet))
        .collect();
    for row in &kept {
        let layer = match row.layer {
            CheckLayer::Scenario => "scenario",
            CheckLayer::Freenet => "freenet",
            CheckLayer::Libp2p => "libp2p",
        };
        report.checks.push(ReportCheck {
            name: format!("{layer} {}", row.name),
            ok: row.pass,
            detail: format!("expected {} | actual {}", row.expected, row.actual),
        });
    }
    let tables = crate::expectation_result::ExpectationResult::render_markdown_tables(&kept);
    if !tables.is_empty() {
        report.caption.push('\n');
        report.caption.push_str(&tables);
    }
    if check_history {
        let written = peer_names.iter().any(|name| {
            let log = report.logs_dir.join(format!("instance-{name}.log"));
            std::fs::read_to_string(&log)
                .unwrap_or_default()
                .contains("room history written")
        });
        report.checks.push(ReportCheck {
            name: "scenario history written".to_string(),
            ok: written,
            detail: if written {
                "some holder logged room history written".to_string()
            } else {
                "no holder logged room history written".to_string()
            },
        });
        report.caption.push('\n');
        report.caption.push_str("scenario history written: ");
        report
            .caption
            .push_str(if written { "PASS" } else { "FAIL" });
    }
}

// needed helper: best-effort clip send, skipped when Telegram has no credentials
async fn send_clip(clip: Option<&std::path::Path>, caption: &str) {
    let Some(clip) = clip else {
        return;
    };
    if telegram_bot_helper::TelegramBot::from_env().is_none() {
        return;
    }
    let clip = clip.to_path_buf();
    let caption = caption.to_string();
    let _ = tokio::task::spawn_blocking(move || {
        telegram_bot_helper::TelegramBot::send_video_best_effort(
            &clip,
            &telegram_bot_helper::Caption::from(caption.as_str()),
        );
    })
    .await;
}

fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}
