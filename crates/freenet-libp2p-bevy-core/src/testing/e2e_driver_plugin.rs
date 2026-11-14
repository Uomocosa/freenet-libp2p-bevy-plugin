use std::net::Ipv4Addr;

use bevy::input::InputSystems;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::remote::RemotePlugin;
use bevy::remote::http::RemoteHttpPlugin;
use bevy::window::{CursorMoved, WindowEvent};

use crate::testing;

pub struct E2EDriverPlugin {
    port: u16,
    extensions: Vec<Box<dyn testing::traits::E2EMethods>>,
}

impl E2EDriverPlugin {
    #[must_use]
    pub fn new(port: u16) -> Self {
        Self {
            port,
            extensions: Vec::new(),
        }
    }
}

impl Plugin for E2EDriverPlugin {
    fn build(&self, app: &mut App) {
        let generic = RemotePlugin::default()
            .with_method_main("e2e/move_cursor", testing::bevy_systems::e2e_move_cursor)
            .with_method_main("e2e/click_at", testing::bevy_systems::e2e_click_at)
            .with_method_main("e2e/follow", testing::bevy_systems::e2e_follow)
            .with_method_main("e2e/type_text", testing::bevy_systems::e2e_type_text)
            .with_method_main("e2e/press_key", testing::bevy_systems::e2e_press_key)
            .with_method_main("e2e/input_idle", testing::bevy_systems::e2e_input_idle)
            .with_method_main("e2e/exit", testing::bevy_systems::e2e_exit);
        let remote = self
            .extensions
            .iter()
            .fold(generic, |remote, extension| extension.register(remote));
        self.extensions
            .iter()
            .for_each(|extension| extension.build(app));
        app.add_plugins(remote)
            .add_plugins(
                RemoteHttpPlugin::default()
                    .with_address(Ipv4Addr::LOCALHOST)
                    .with_port(self.port),
            )
            .init_resource::<testing::SyntheticInputQueue>()
            .add_message::<CursorMoved>()
            .add_message::<WindowEvent>()
            .add_message::<MouseButtonInput>()
            .add_message::<KeyboardInput>()
            .add_systems(
                PreUpdate,
                testing::bevy_systems::drain_synthetic_input.before(InputSystems),
            );
    }
}

impl E2EDriverPlugin {
    #[must_use]
    pub fn with(mut self, extension: impl testing::traits::E2EMethods) -> Self {
        self.extensions.push(Box::new(extension));
        self
    }
}

#[cfg(test)]
mod tests {
    use bevy::remote::RemotePlugin;

    use super::E2EDriverPlugin;
    use crate::testing;

    struct Probe;

    impl testing::traits::E2EMethods for Probe {
        fn register(&self, remote: RemotePlugin) -> RemotePlugin {
            remote
        }
    }

    #[test]
    fn test_usage() {
        let plugin = E2EDriverPlugin::new(15702).with(Probe).with(Probe);
        assert_eq!(plugin.port, 15702);
        assert_eq!(plugin.extensions.len(), 2);
    }
}
