mod common;

fn peer_count() -> usize {
    std::env::var("E2E_PEERS")
        .ok()
        .and_then(|raw| raw.parse().ok())
        .unwrap_or(3)
        .clamp(2, 5)
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e: needs X + freenet network + N lobby_room windows"]
#[telegram_bot_helper::telegram_notify]
async fn lobby_room_discovery() {
    common::run_scenario(common::Scenario::discover_only(peer_count()))
        .await
        .assert_all();
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e: needs X + freenet network + N lobby_room windows"]
#[telegram_bot_helper::telegram_notify]
async fn freenet_ring() {
    common::run_scenario(common::Scenario::ring_only(peer_count()))
        .await
        .assert_all();
}
