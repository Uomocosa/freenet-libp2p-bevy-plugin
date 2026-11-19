use bevy::prelude::*;
use bevy::remote::RemotePlugin;

pub trait E2EMethods: Send + Sync + 'static {
    fn register(&self, remote: RemotePlugin) -> RemotePlugin;
    fn build(&self, _app: &mut App) {}
}

#[cfg(test)]
mod tests {
    use bevy::prelude::App;

    use crate::testing;
    use crate::testing::traits::E2EMethods;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        testing::Probe.build(&mut app);
        assert!(
            app.world()
                .contains_resource::<testing::SyntheticInputQueue>()
        );
    }
}
