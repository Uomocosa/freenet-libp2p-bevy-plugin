use bevy::remote::RemotePlugin;

use crate::testing;

pub fn e2e_methods_register(
    _direct_dial_e2e_methods: &testing::DirectDialE2EMethods,
    remote: RemotePlugin,
) -> RemotePlugin {
    remote
        .with_method_main("e2e/dial", testing::bevy_systems::e2e_dial)
        .with_method_main(
            "direct_dial/status",
            testing::bevy_systems::direct_dial_status,
        )
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{App, MinimalPlugins};
    use bevy::remote::{RemoteMethods, RemotePlugin};

    use super::e2e_methods_register;
    use crate::testing;

    #[test]
    fn test_usage() {
        let registered = {
            let remote =
                e2e_methods_register(&testing::DirectDialE2EMethods, RemotePlugin::default());
            let mut app = App::new();
            app.add_plugins(MinimalPlugins).add_plugins(remote);
            app.world().resource::<RemoteMethods>().methods()
        };
        assert!(registered.contains(&String::from("e2e/dial")));
        assert!(registered.contains(&String::from("direct_dial/status")));
    }
}
