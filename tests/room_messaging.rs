mod common;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e: needs X + freenet network + N lobby_room windows"]
#[telegram_bot_helper::telegram_notify]
async fn room_messaging() {
    common::run_scenario(common::Scenario::room_messaging(3))
        .await
        .assert_all();
}
