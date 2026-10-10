mod common;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e: needs X + freenet network + N lobby_room windows"]
#[telegram_bot_helper::telegram_notify]
async fn lobby_room_discovery() {
    common::run_scenario(common::Scenario::discover_only(3))
        .await
        .assert_all();
}
