use bevy::remote::RemotePlugin;

use crate::testing;

pub fn e2e_methods_register(
    _lobby_e2e_methods: &testing::LobbyE2EMethods,
    remote: RemotePlugin,
) -> RemotePlugin {
    remote
        .with_method_main("lobby/status", testing::bevy_systems::lobby_status)
        .with_method_main("lobby/locate", testing::bevy_systems::lobby_locate)
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
            let remote = e2e_methods_register(&testing::LobbyE2EMethods, RemotePlugin::default());
            let mut app = App::new();
            app.add_plugins(MinimalPlugins).add_plugins(remote);
            app.world().resource::<RemoteMethods>().methods()
        };
        assert!(registered.contains(&String::from("lobby/status")));
        assert!(registered.contains(&String::from("lobby/locate")));
    }
}
