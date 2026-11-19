use std::time::Duration;

use bevy::prelude::App;

use crate::identity;
use crate::testing;

#[must_use]
pub fn wait_ready(app: &mut App, rounds: usize, settle: Duration) -> Option<identity::Peer> {
    if testing::wait_until(rounds, settle, |_| {
        app.update();
        testing::ready_peer(app).is_some()
    }) {
        return testing::ready_peer(app);
    }
    None
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::p2p;
    use crate::testing;

    use super::wait_ready;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        app.insert_resource(p2p::ReadyChannel::default());
        assert!(wait_ready(&mut app, 1, Duration::ZERO).is_none());
        let peer = testing::peer("a", &[]);
        app.world()
            .resource::<p2p::ReadyChannel>()
            .ready_tx
            .send(Some(peer.clone()))
            .expect("send");
        assert_eq!(wait_ready(&mut app, 1, Duration::ZERO), Some(peer));
    }
}
