use bevy::prelude::App;

use crate::starboard::app::test_harness::{MAX_ROUNDS, SETTLE};
use derive_more::{Deref, DerefMut};
use freenet_libp2p_bevy_plugin::{identity, p2p};

#[derive(Deref, DerefMut)]
pub struct ConnectionEvents(pub tokio::sync::broadcast::Receiver<p2p::Libp2pEvent>);

impl ConnectionEvents {
    pub(crate) fn wait_connected(&mut self, app: &mut App, peer: &identity::PeerId) {
        for _ in 0..MAX_ROUNDS {
            app.update();
            while let Ok(event) = self.try_recv() {
                if let p2p::Libp2pEvent::PeerConnected(id) = event
                    && &id == peer
                {
                    return;
                }
            }
            std::thread::sleep(SETTLE);
        }
        panic!("peer never connected");
    }
}

// no test_usage necessary
