use crate::discovery;

#[derive(Debug, Clone)]
pub struct Config {
    pub game_name: discovery::GameName,
    pub token: discovery::GameToken,
    pub timing: discovery::Timing,
    pub capacity: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            game_name: discovery::GameName::from("test"),
            token: discovery::GameToken::from("test"),
            timing: discovery::Timing::default(),
            capacity: 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let config = Config::default();
        let params = discovery::freenet::contract_params(&config.game_name, &config.token);
        assert_eq!(params, b"test/test".to_vec());
        assert_eq!(config.capacity, 8);
    }
}
