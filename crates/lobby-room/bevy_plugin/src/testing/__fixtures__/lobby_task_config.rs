use crate::discovery;
use crate::identity;

#[must_use]
pub fn lobby_task_config(me: &str) -> discovery::LobbyTaskConfig {
    discovery::LobbyTaskConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        me: identity::PeerId::from(me),
        params: Vec::new(),
        discovery: discovery::Config::default(),
    }
}

#[cfg(test)]
mod tests {
    use crate::identity;

    use super::lobby_task_config;

    #[test]
    fn test_usage() {
        let config = lobby_task_config("me");
        assert_eq!(config.me, identity::PeerId::from("me"));
        assert_eq!(config.port, 0);
    }
}
