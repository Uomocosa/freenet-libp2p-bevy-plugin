mod common;

use std::path::Path;
use std::time::Duration;

fn relaunch_gap_secs() -> u64 {
    std::env::var("E2E_RELAUNCH_GAP_SECS")
        .ok()
        .and_then(|raw| raw.parse().ok())
        .unwrap_or(10)
}

fn relaunch_stamp_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e: needs X + freenet network + 1 lobby_room window, twice with the same node folder"]
#[telegram_bot_helper::telegram_notify]
async fn freenet_relaunch() {
    let stamp = relaunch_stamp_secs();
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".local-run")
        .join(format!("freenet_relaunch-{stamp}"));
    assert!(
        std::fs::create_dir_all(&base).is_ok(),
        "create shared run dir {}",
        base.display()
    );
    let node_dir = base.join("node");
    assert!(
        std::fs::create_dir_all(&node_dir).is_ok(),
        "create shared node dir {}",
        node_dir.display()
    );
    let token = format!("lobby-e2e/{stamp}");
    let cold = common::Scenario::relaunch(
        "freenet_relaunch_cold".to_string(),
        token.clone(),
        node_dir.clone(),
    );
    common::run_scenario(cold).await.assert_all();
    tokio::time::sleep(Duration::from_secs(relaunch_gap_secs())).await;
    let warm = common::Scenario::relaunch("freenet_relaunch_warm".to_string(), token, node_dir);
    common::run_scenario(warm).await.assert_all();
}
