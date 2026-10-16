#[macro_export]
macro_rules! game_id {
    () => {
        $crate::identity::GameId(format!(
            "{}/{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_REPOSITORY"),
        ))
    };
}
// no test_usage necessary
