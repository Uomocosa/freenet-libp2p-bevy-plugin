use std::str::FromStr;

use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Create(identity::RoomName),
    Join(identity::RoomName),
}

impl FromStr for Action {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let (kind, room) = raw
            .split_once(':')
            .ok_or_else(|| format!("expected create:<room> or join:<room>, got {raw}"))?;
        let room = Self::checked_room(room)?;
        match kind {
            "create" => Ok(Self::Create(room)),
            "join" => Ok(Self::Join(room)),
            other => Err(format!("expected create or join, got {other}")),
        }
    }
}

impl Action {
    pub(crate) fn checked_room(room: &str) -> Result<identity::RoomName, String> {
        if room.trim().is_empty() {
            return Err("room name is empty".to_string());
        }
        Ok(identity::RoomName::from(room))
    }
}

#[cfg(test)]
mod tests {
    use super::Action;
    use crate::identity;

    #[test]
    fn test_usage() {
        assert!(matches!(
            "create:alpha".parse::<Action>(),
            Ok(Action::Create(room)) if room == identity::RoomName::from("alpha")
        ));
        assert!(matches!(
            "join:beta".parse::<Action>(),
            Ok(Action::Join(room)) if room == identity::RoomName::from("beta")
        ));
        assert!("create".parse::<Action>().is_err());
        assert!("create:".parse::<Action>().is_err());
        assert!("create:   ".parse::<Action>().is_err());
        assert!("frob:alpha".parse::<Action>().is_err());
    }
}
