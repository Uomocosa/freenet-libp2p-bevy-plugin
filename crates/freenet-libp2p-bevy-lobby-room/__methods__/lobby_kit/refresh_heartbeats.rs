use std::collections::BTreeMap;

use crate::discovery;
use crate::identity;
use crate::testing;
use crate::time;

pub fn refresh_heartbeats(
    kit: &mut testing::LobbyKit,
    peer: &identity::PeerId,
    game: &identity::GameId,
    targets: &[discovery::state_machine::HeartbeatTarget],
    now: time::UnixTime,
) -> discovery::Lobby {
    kit.heartbeat_targets
        .entry(game.clone())
        .or_default()
        .insert(peer.clone(), targets.to_owned());
    let mut fresh = discovery::Lobby::default();
    for target in targets {
        let record = fresh
            .entry(target.room.clone())
            .or_insert_with(|| discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: target.created_at,
                members: BTreeMap::new(),
            });
        if target.created_at < record.created_at {
            record.created_at = target.created_at;
        }
        record.members.insert(
            peer.clone(),
            discovery::RoomMember {
                peer: identity::Peer {
                    id: peer.clone(),
                    addrs: target.addrs.clone(),
                },
                updated_at: now,
                display_name: Some(target.display_name.clone()),
                membership: target.membership,
            },
        );
    }
    fresh.drop_rooms_where_every_member_quit();
    let shared = kit.shared_lobbies.entry(game.clone()).or_default();
    shared.merge(fresh);
    shared.clone()
}
#[cfg(test)]
mod tests {
    use super::refresh_heartbeats;
    use crate::discovery;
    use crate::identity;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut kit = testing::LobbyKit::default();
        let lobby = refresh_heartbeats(
            &mut kit,
            &identity::PeerId::from("a"),
            &identity::GameId::from("game"),
            &[],
            time::UnixTime::from_secs(1),
        );
        assert!(lobby.is_empty());
        let _ = discovery::Lobby::default();
    }
}
