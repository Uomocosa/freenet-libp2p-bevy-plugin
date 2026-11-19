use bevy::prelude::App;

use crate::identity;
use crate::p2p;

#[must_use]
pub fn ready_peer(app: &App) -> Option<identity::Peer> {
    app.world()
        .get_resource::<p2p::ReadyChannel>()?
        .ready_rx
        .borrow()
        .clone()
}

#[cfg(test)]
mod tests {
    use crate::p2p;
    use crate::testing;

    use super::ready_peer;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        app.insert_resource(p2p::ReadyChannel::default());
        assert!(ready_peer(&app).is_none());
        let peer = testing::peer("a", &[]);
        app.world()
            .resource::<p2p::ReadyChannel>()
            .ready_tx
            .send(Some(peer.clone()))
            .expect("send");
        assert_eq!(ready_peer(&app), Some(peer));
    }
}
