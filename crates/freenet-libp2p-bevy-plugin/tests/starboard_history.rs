mod common;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e: needs X + freenet network + N starboard windows"]
#[telegram_bot_helper::telegram_notify]
async fn starboard_history_restored_after_everyone_left_the_room() {
    common::run_scenario(common::Scenario::starboard_history_restore(3))
        .await
        .assert_all();
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e: needs X + freenet network + N starboard windows"]
#[telegram_bot_helper::telegram_notify]
async fn starboard_room_gone_when_every_player_quit() {
    common::run_scenario(common::Scenario::starboard_history_room_gone(4))
        .await
        .assert_all();
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e: needs X + freenet network + N starboard windows"]
#[telegram_bot_helper::telegram_notify]
async fn starboard_joiner_sees_the_error_screen_when_the_last_holder_is_killed() {
    common::run_scenario(common::Scenario::starboard_history_holder_killed(2))
        .await
        .assert_all();
}
