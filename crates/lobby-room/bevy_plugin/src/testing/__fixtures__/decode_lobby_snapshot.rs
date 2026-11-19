use crate::discovery;

pub fn decode_lobby_snapshot(frames: &[Vec<u8>]) -> Result<discovery::Lobby, bincode::Error> {
    let frame = frames
        .first()
        .ok_or_else(|| Box::new(bincode::ErrorKind::Custom("no lobby frame".to_string())))?;
    match bincode::deserialize::<discovery::LobbyWire>(frame)? {
        discovery::LobbyWire::Snapshot(lobby) => Ok(lobby),
        discovery::LobbyWire::HeartbeatTargets(_) => Err(Box::new(bincode::ErrorKind::Custom(
            "expected a lobby snapshot".to_string(),
        ))),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::discovery;
    use crate::time;

    use super::decode_lobby_snapshot;

    #[test]
    fn test_usage() {
        let lobby = discovery::Lobby::from([(
            discovery::RoomName::from("room-a"),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::EPOCH,
                members: BTreeMap::new(),
            },
        )]);
        let frame =
            bincode::serialize(&discovery::LobbyWire::Snapshot(lobby)).expect("encode snapshot");
        let decoded = decode_lobby_snapshot(&[frame]).expect("snapshot");
        assert!(decoded.contains_key(&discovery::RoomName::from("room-a")));
    }
}
