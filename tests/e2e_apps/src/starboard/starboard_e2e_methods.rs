use bevy::prelude::*;
use bevy::remote::RemotePlugin;
use freenet_libp2p_bevy_plugin::testing::traits::E2EMethods;

use crate::starboard::{CanvasHashWatch, bevy_systems};

pub struct StarboardE2EMethods;

impl E2EMethods for StarboardE2EMethods {
    #[rustfmt::skip]
    fn register(&self, remote: RemotePlugin) -> RemotePlugin { remote.with_method_main("starboard/status", bevy_systems::starboard_status).with_method_main("starboard/board_to_window", bevy_systems::starboard_board_to_window) }

    fn build(&self, app: &mut App) {
        app.init_resource::<CanvasHashWatch>()
            .add_systems(Update, bevy_systems::record_canvas_hash);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::remote::{RemoteMethods, RemotePlugin};
    use freenet_libp2p_bevy_plugin::testing::traits::E2EMethods;

    use super::StarboardE2EMethods;
    use crate::starboard::CanvasHashWatch;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(StarboardE2EMethods.register(RemotePlugin::default()));
        StarboardE2EMethods.build(&mut app);
        let methods = app.world().resource::<RemoteMethods>();
        assert!(methods.get("starboard/status").is_some());
        assert!(methods.get("starboard/board_to_window").is_some());
        assert!(app.world().contains_resource::<CanvasHashWatch>());
    }
}
