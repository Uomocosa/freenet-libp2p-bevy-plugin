use std::time::Duration;

use bevy::prelude::App;

use crate::p2p;
use crate::testing;

#[must_use]
pub fn wait_for_history(
    player: &mut App,
    events: &mut tokio::sync::broadcast::Receiver<p2p::Libp2pEvent>,
    prefix: &p2p::HistoryKeyPrefix,
    rounds: usize,
    settle: Duration,
) -> bool {
    testing::wait_until(rounds, settle, |_| {
        player.update();
        while let Ok(event) = events.try_recv() {
            if let p2p::Libp2pEvent::HistoryRecordStored {
                id: p2p::ZoneHistoryRecordKind::Manifest(stored),
            } = event
                && &stored == prefix
            {
                return true;
            }
        }
        false
    })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::p2p;
    use crate::testing;

    use super::wait_for_history;

    #[test]
    fn test_usage() {
        let mut player = testing::TestApp::new();
        player.insert_resource(p2p::Libp2pEventChannel::default());
        let mut events = player
            .world()
            .resource::<p2p::Libp2pEventChannel>()
            .subscribe();
        let prefix = p2p::HistoryKeyPrefix::from("prefix");
        player
            .world()
            .resource::<p2p::Libp2pEventChannel>()
            .tx
            .send(p2p::Libp2pEvent::HistoryRecordStored {
                id: p2p::ZoneHistoryRecordKind::Manifest(prefix.clone()),
            })
            .expect("send");
        assert!(wait_for_history(
            &mut player,
            &mut events,
            &prefix,
            1,
            Duration::ZERO
        ));
        assert!(!wait_for_history(
            &mut player,
            &mut events,
            &p2p::HistoryKeyPrefix::from("other"),
            1,
            Duration::ZERO
        ));
    }
}
