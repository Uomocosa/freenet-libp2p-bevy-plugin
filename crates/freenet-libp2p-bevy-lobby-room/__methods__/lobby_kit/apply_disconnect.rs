use crate::discovery;
use crate::identity;
use crate::testing;

pub fn apply_disconnect(
    kit: &mut testing::LobbyKit,
    peer: &identity::PeerId,
    game: &identity::GameId,
) -> Vec<testing::FanOut> {
    if let Some(heartbeats) = kit.heartbeat_targets.get_mut(game) {
        heartbeats.remove(peer);
    }
    if let Some(lobby) = kit.shared_lobbies.get_mut(game) {
        for room in lobby.values_mut() {
            let is_departed = room
                .members
                .get(peer)
                .is_some_and(|member| member.membership == discovery::RoomMembership::Departed);
            if is_departed {
                room.members.remove(peer);
            }
        }
        lobby.drop_rooms_where_every_member_quit();
    }
    Vec::new()
}
#[cfg(test)]
mod tests {
    use super::apply_disconnect;
    use crate::identity;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut kit = testing::LobbyKit::default();
        let fan_outs = apply_disconnect(
            &mut kit,
            &identity::PeerId::from("a"),
            &identity::GameId::from("game"),
        );
        assert_eq!(fan_outs, Vec::new());
    }
}
