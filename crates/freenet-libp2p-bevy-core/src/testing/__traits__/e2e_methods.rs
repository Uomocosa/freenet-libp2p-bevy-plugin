use bevy::prelude::*;
use bevy::remote::RemotePlugin;

pub trait E2EMethods: Send + Sync + 'static {
    fn register(&self, remote: RemotePlugin) -> RemotePlugin;
    fn build(&self, _app: &mut App) {}
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::remote::RemotePlugin;

    use super::E2EMethods;

    #[derive(Resource)]
    struct Probed;

    struct Probe;

    impl E2EMethods for Probe {
        fn register(&self, remote: RemotePlugin) -> RemotePlugin {
            remote
        }

        fn build(&self, app: &mut App) {
            app.insert_resource(Probed);
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        Probe.build(&mut app);
        assert!(app.world().contains_resource::<Probed>());
    }
}
