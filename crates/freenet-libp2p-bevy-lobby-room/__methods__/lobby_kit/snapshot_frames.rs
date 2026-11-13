use crate::discovery;
use crate::identity;
use crate::testing;

pub fn snapshot_frames(kit: &testing::LobbyKit, game: &identity::GameId) -> Vec<Vec<u8>> {
    let lobby = kit.shared_lobbies.get(game).cloned().unwrap_or_default();
    bincode::serialize(&discovery::LobbyWire::Snapshot(lobby))
        .into_iter()
        .collect()
}
#[cfg(test)]
mod tests {
    use super::snapshot_frames;
    use crate::identity;
    use crate::testing;

    #[test]
    fn test_usage() {
        let kit = testing::LobbyKit::default();
        assert_eq!(
            snapshot_frames(&kit, &identity::GameId::from("game")).len(),
            1
        );
    }
}
