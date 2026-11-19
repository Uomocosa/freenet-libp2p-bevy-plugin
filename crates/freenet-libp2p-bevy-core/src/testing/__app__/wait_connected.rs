use std::time::Duration;

use bevy::prelude::App;

use crate::identity;
use crate::p2p;
use crate::testing;

#[must_use]
pub fn wait_connected(
    app: &mut App,
    events: &mut tokio::sync::broadcast::Receiver<p2p::Libp2pEvent>,
    peer: &identity::PeerId,
    rounds: usize,
    settle: Duration,
) -> bool {
    testing::wait_until(rounds, settle, |_| {
        app.update();
        while let Ok(event) = events.try_recv() {
            if let p2p::Libp2pEvent::PeerConnected(id) = event
                && &id == peer
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

    use crate::identity;
    use crate::p2p;
    use crate::testing;

    use super::wait_connected;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        app.insert_resource(p2p::Libp2pEventChannel::default());
        let mut events = app
            .world()
            .resource::<p2p::Libp2pEventChannel>()
            .subscribe();
        let peer = identity::PeerId::from("a");
        app.world()
            .resource::<p2p::Libp2pEventChannel>()
            .tx
            .send(p2p::Libp2pEvent::PeerConnected(peer.clone()))
            .expect("send");
        assert!(wait_connected(
            &mut app,
            &mut events,
            &peer,
            1,
            Duration::ZERO
        ));
        assert!(!wait_connected(
            &mut app,
            &mut events,
            &identity::PeerId::from("other"),
            1,
            Duration::ZERO
        ));
    }
}
