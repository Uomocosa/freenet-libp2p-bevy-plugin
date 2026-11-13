use crate::discovery;
use crate::identity;
use crate::testing;
use crate::time;

pub fn apply_extension(
    kit: &mut testing::LobbyKit,
    from: &identity::PeerId,
    game: &identity::GameId,
    bytes: &[u8],
    now: time::UnixTime,
) -> Vec<testing::FanOut> {
    let Ok(wire) = bincode::deserialize::<discovery::LobbyWire>(bytes) else {
        tracing::error!(target: "testing", "fake lobby: extension decode failed");
        return Vec::new();
    };
    let discovery::LobbyWire::HeartbeatTargets(targets) = wire else {
        return Vec::new();
    };
    let lobby = kit.refresh_heartbeats(from, game, &targets, now);
    bincode::serialize(&discovery::LobbyWire::Snapshot(lobby))
        .map(|bytes| testing::FanOut {
            to: testing::FanOutTarget::Game,
            bytes,
        })
        .into_iter()
        .collect()
}
#[cfg(test)]
mod tests {
    use super::apply_extension;
    use crate::identity;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut kit = testing::LobbyKit::default();
        let fan_outs = apply_extension(
            &mut kit,
            &identity::PeerId::from("a"),
            &identity::GameId::from("game"),
            &[0xff],
            time::UnixTime::from_secs(1),
        );
        assert_eq!(fan_outs, Vec::new());
    }
}
