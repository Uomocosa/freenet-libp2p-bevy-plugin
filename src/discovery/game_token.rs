#[macro_export]
macro_rules! game_token {
    () => {
        $crate::discovery::GameToken(format!(
            "{}/{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_REPOSITORY"),
        ))
    };
}
// no test_usage necessary
