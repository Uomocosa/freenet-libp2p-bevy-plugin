use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LobbyRoomTarget {
    NameField,
    NewRoomButton,
    Join(String),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::LobbyRoomTarget;

    #[test]
    fn test_usage() {
        let value = serde_json::to_value(LobbyRoomTarget::Join("alpha".to_string())).expect("s");
        assert_eq!(value, json!({ "Join": "alpha" }));
        let back: LobbyRoomTarget = serde_json::from_value(json!("NameField")).expect("d");
        assert_eq!(back, LobbyRoomTarget::NameField);
    }
}
