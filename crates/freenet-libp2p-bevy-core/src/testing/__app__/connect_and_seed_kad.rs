use std::time::Duration;

use bevy::prelude::App;

use crate::identity;
use crate::p2p;
use crate::testing;

#[must_use]
pub fn connect_and_seed_kad(
    app: &mut App,
    peer: &identity::Peer,
    rounds: usize,
    settle: Duration,
) -> bool {
    let mut events = app
        .world()
        .resource::<p2p::Libp2pEventChannel>()
        .subscribe();
    let sender = app.world().resource::<p2p::Libp2pCommandSender>();
    let _ = sender.send(p2p::Libp2pCommand::AddKadPeer(peer.clone()));
    let _ = sender.send(p2p::Libp2pCommand::Dial(peer.clone()));
    testing::wait_connected(app, &mut events, &peer.id, rounds, settle)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::p2p;
    use crate::testing;

    use super::connect_and_seed_kad;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        app.insert_resource(p2p::Libp2pEventChannel::default());
        let (sender, mut commands) = tokio::sync::mpsc::unbounded_channel();
        app.insert_resource(p2p::Libp2pCommandSender(sender));
        let peer = testing::peer("a", &[]);
        assert!(!connect_and_seed_kad(&mut app, &peer, 1, Duration::ZERO));
        assert!(matches!(
            commands.try_recv(),
            Ok(p2p::Libp2pCommand::AddKadPeer(got)) if got.id == peer.id
        ));
        assert!(matches!(
            commands.try_recv(),
            Ok(p2p::Libp2pCommand::Dial(got)) if got.id == peer.id
        ));
    }
}
