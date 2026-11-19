use bevy::prelude::{App, Entity, With};

use crate::spectator;
use crate::testing;

pub fn despawn_spectated(app: &mut App) {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<spectator::Spectated<testing::FakeReplica<u8, u8>>>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    for entity in entities {
        app.world_mut().despawn(entity);
    }
}

#[cfg(test)]
mod tests {
    use crate::spectator;
    use crate::testing;

    use super::despawn_spectated;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        app.world_mut().spawn(testing::spectated_zone(
            spectator::SpectateSlot::Preview,
            "alpha",
            spectator::SpectateFreshness::Live,
        ));
        despawn_spectated(&mut app);
        assert!(
            testing::find_entity::<spectator::Spectated<testing::FakeReplica<u8, u8>>>(
                app.world_mut()
            )
            .is_none()
        );
    }
}
