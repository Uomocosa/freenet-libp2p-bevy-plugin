mod common;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e: needs X + freenet network + N starboard_gossip windows"]
#[telegram_bot_helper::telegram_notify]
async fn starboard_e2e() {
    common::run_scenario(common::Scenario::starboard(3))
        .await
        .assert_all();
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e: needs X + freenet network + N starboard_gossip windows"]
#[telegram_bot_helper::telegram_notify]
async fn starboard_late_join() {
    common::run_scenario(common::Scenario::starboard_late_join(4))
        .await
        .assert_all();
}
