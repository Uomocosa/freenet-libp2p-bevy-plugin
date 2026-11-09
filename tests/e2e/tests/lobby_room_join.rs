mod common;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e: needs X + freenet network + N lobby_room windows"]
#[telegram_bot_helper::telegram_notify]
async fn lobby_room_join() {
    common::run_scenario(common::Scenario::join(5))
        .await
        .assert_all();
}
