use std::time::Duration;

use bevy::prelude::{Assets, Image};

use crate::discovery;
use crate::identity;
use crate::p2p;
use crate::peek;
use crate::testing;
use crate::ui;

#[must_use]
pub fn peek_app(me: &str, rooms: &[(&str, u64)]) -> testing::TestApp {
    let lobby = testing::lobby_with(
        rooms
            .iter()
            .map(|(name, created_at)| {
                (
                    *name,
                    testing::room(discovery::RoomSize::DEFAULT, *created_at, Vec::new()),
                )
            })
            .collect(),
    );
    let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from(me))
        .with_lobby(lobby)
        .build();
    let mut app = testing::TestApp::new().with_frame(Duration::from_millis(50));
    app.add_message::<p2p::Libp2pCommand>();
    app.insert_resource(ui::discovery::SelectedRoom::default());
    app.insert_resource(ui::discovery::StreamedRoom::default());
    app.insert_resource(state);
    app.init_resource::<Assets<Image>>();
    app.with_plugin(peek::RoomPeekPlugin::<testing::FakeReplica<u8, u8>>::new(
        testing::peek_game(),
    ))
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::peek;

    use super::peek_app;

    #[test]
    fn test_usage() {
        let app = peek_app("me", &[("alpha", 10), ("beta", 20)]);
        assert_eq!(
            app.world()
                .resource::<discovery::state_machine::DiscoveryState>()
                .lobby
                .len(),
            2
        );
        assert!(app.world().get_resource::<peek::RoomPeekGame>().is_some());
    }
}
