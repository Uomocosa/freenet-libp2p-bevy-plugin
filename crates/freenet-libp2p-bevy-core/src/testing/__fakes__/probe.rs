use bevy::prelude::App;
use bevy::remote::RemotePlugin;

use crate::testing;

pub struct Probe;

#[rustfmt::skip]
impl testing::traits::E2EMethods for Probe {
    fn register(&self, remote: RemotePlugin) -> RemotePlugin {
        remote
    }

    fn build(&self, app: &mut App) {
        app.init_resource::<testing::SyntheticInputQueue>();
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::App;

    use crate::testing;
    use crate::testing::traits::E2EMethods;

    use super::Probe;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        Probe.build(&mut app);
        assert!(
            app.world()
                .contains_resource::<testing::SyntheticInputQueue>()
        );
    }
}
