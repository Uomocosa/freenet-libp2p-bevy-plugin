use atomic_delegate_macros::atomic_delegate;
use bevy::remote::RemotePlugin;

use crate::testing;

pub struct LobbyE2EMethods;

impl testing::traits::E2EMethods for LobbyE2EMethods {
    #[atomic_delegate(LobbyE2EMethods, E2EMethods)]
    fn register(&self, remote: RemotePlugin) -> RemotePlugin {}
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::remote::{RemoteMethods, RemotePlugin};

    use crate::testing;
    use crate::testing::traits::E2EMethods;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(testing::LobbyE2EMethods.register(RemotePlugin::default()));
        let methods = app.world().resource::<RemoteMethods>();
        assert!(methods.get("lobby/status").is_some());
        assert!(methods.get("lobby/locate").is_some());
    }
}
