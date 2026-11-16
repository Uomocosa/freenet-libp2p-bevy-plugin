use bevy::prelude::*;
use bevy::remote::RemotePlugin;
use freenet_libp2p_bevy_core::testing::traits::E2EMethods;

use crate::bevy_systems;
use crate::input_recorder::InputRecorder;

pub struct ProbeE2EMethods;

impl E2EMethods for ProbeE2EMethods {
    #[rustfmt::skip]
    fn register(&self, remote: RemotePlugin) -> RemotePlugin {
        remote.with_method_main("probe/records", bevy_systems::probe_records).with_method_main("probe/clear", bevy_systems::probe_clear).with_method_main("probe/ready", bevy_systems::probe_ready).with_method_main("probe/window", bevy_systems::probe_window)
    }

    fn build(&self, app: &mut App) {
        app.init_resource::<InputRecorder>()
            .add_systems(Update, bevy_systems::record_input);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::remote::{RemoteMethods, RemotePlugin};
    use freenet_libp2p_bevy_core::testing::traits::E2EMethods;

    use super::ProbeE2EMethods;
    use crate::input_recorder::InputRecorder;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(ProbeE2EMethods.register(RemotePlugin::default()));
        let methods = app.world().resource::<RemoteMethods>();
        assert!(methods.get("probe/records").is_some());
        assert!(methods.get("probe/clear").is_some());
        assert!(methods.get("probe/ready").is_some());
        assert!(methods.get("probe/window").is_some());
        ProbeE2EMethods.build(&mut app);
        assert!(app.world().contains_resource::<InputRecorder>());
    }
}
