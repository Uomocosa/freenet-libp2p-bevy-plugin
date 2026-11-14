use bevy::remote::RemotePlugin;
use freenet_libp2p_bevy_plugin::testing::traits::E2EMethods;

use crate::lobby_room::bevy_systems;

pub struct LobbyRoomE2EMethods;

impl E2EMethods for LobbyRoomE2EMethods {
    #[rustfmt::skip]
    fn register(&self, remote: RemotePlugin) -> RemotePlugin { remote.with_method_main("lobby_room/locate", bevy_systems::lobby_room_locate).with_method_main("lobby_room/messages", bevy_systems::lobby_room_messages) }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::remote::{RemoteMethods, RemotePlugin};
    use freenet_libp2p_bevy_plugin::testing::traits::E2EMethods;

    use super::LobbyRoomE2EMethods;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(LobbyRoomE2EMethods.register(RemotePlugin::default()));
        let methods = app.world().resource::<RemoteMethods>();
        assert!(methods.get("lobby_room/locate").is_some());
        assert!(methods.get("lobby_room/messages").is_some());
    }
}
