use bevy::remote::RemotePlugin;
use freenet_libp2p_bevy_core::testing::traits::E2EMethods;

pub struct ProbeE2EMethods;

impl E2EMethods for ProbeE2EMethods {
    fn register(&self, remote: RemotePlugin) -> RemotePlugin {
        remote
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::remote::RemotePlugin;

    use super::ProbeE2EMethods;
    use freenet_libp2p_bevy_core::testing::traits::E2EMethods;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(ProbeE2EMethods.register(RemotePlugin::default()));
        ProbeE2EMethods.build(&mut app);
    }
}
