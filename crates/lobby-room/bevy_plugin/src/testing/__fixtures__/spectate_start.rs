use crate::discovery;
use crate::p2p;
use crate::spectator;
use crate::testing;
use crate::time;

#[must_use]
pub fn spectate_start(
    slot: spectator::SpectateSlot,
    room: &str,
    created_at: u64,
) -> spectator::SpectateCommand {
    let room = discovery::RoomName::from(room);
    spectator::SpectateCommand::Start {
        slot,
        zone: (*room).clone(),
        history_prefix: p2p::HistoryKeyPrefix::new(
            &testing::peek_game(),
            &room,
            time::UnixTime::from_secs(created_at),
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::spectator;
    use crate::testing;
    use crate::time;

    use super::spectate_start;

    #[test]
    fn test_usage() {
        let command = spectate_start(spectator::SpectateSlot::Preview, "alpha", 10);
        assert_eq!(
            command,
            spectator::SpectateCommand::Start {
                slot: spectator::SpectateSlot::Preview,
                zone: identity::ZoneName::from("alpha"),
                history_prefix: p2p::HistoryKeyPrefix::new(
                    &testing::peek_game(),
                    &discovery::RoomName::from("alpha"),
                    time::UnixTime::from_secs(10),
                ),
            }
        );
    }
}
