mod common;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e: needs X + freenet network + N starboard windows"]
#[telegram_bot_helper::telegram_notify]
async fn starboard_leave_rejoin() {
    common::run_scenario(common::Scenario::starboard_leave_rejoin(3))
        .await
        .assert_all();
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e: needs X + freenet network + N starboard windows"]
#[telegram_bot_helper::telegram_notify]
async fn starboard_simultaneous_paint() {
    common::run_scenario(common::Scenario::starboard_simultaneous_paint(2))
        .await
        .assert_all();
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e: needs X + freenet network + N starboard windows"]
#[telegram_bot_helper::telegram_notify]
async fn starboard_peer_killed() {
    common::run_scenario(common::Scenario::starboard_peer_killed(3))
        .await
        .assert_all();
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e: needs X + freenet network + N starboard windows"]
#[telegram_bot_helper::telegram_notify]
async fn starboard_bad_token() {
    common::run_scenario(common::Scenario::starboard_bad_token(3))
        .await
        .assert_all();
}
